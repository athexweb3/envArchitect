use resolvo::{
    Candidates, Condition, ConditionId, Dependencies, DependencyProvider, Interner,
    KnownDependencies, NameId, SolvableId, SolverCache, StringId, VersionSetId, VersionSetUnionId,
};
use semver::{Version, VersionReq};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// A concrete package version in our system
#[derive(Debug, Clone, Eq)]
pub struct SolverPackage {
    pub name: String,
    pub version: Version,
    pub deps: HashMap<String, VersionReq>,
}

impl PartialEq for SolverPackage {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl Hash for SolverPackage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.version.hash(state);
    }
}

impl Display for SolverPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// The Engine that drives the SAT resolution.
pub struct SatEngine {
    pub registry: HashMap<String, Vec<SolverPackage>>,

    strings: RefCell<Vec<String>>,
    string_to_id: RefCell<HashMap<String, u32>>,
    names: RefCell<Vec<String>>,
    name_to_id: RefCell<HashMap<String, NameId>>,

    version_sets: RefCell<Vec<(NameId, VersionReq)>>,

    solvables: RefCell<Vec<SolverPackage>>,
}

impl SatEngine {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            strings: RefCell::new(Vec::new()),
            string_to_id: RefCell::new(HashMap::new()),
            names: RefCell::new(Vec::new()),
            name_to_id: RefCell::new(HashMap::new()),
            version_sets: RefCell::new(Vec::new()),
            solvables: RefCell::new(Vec::new()),
        }
    }

    pub fn add_package(&mut self, pkg: SolverPackage) {
        self.registry.entry(pkg.name.clone()).or_default().push(pkg);
    }

    pub fn load_registry(&self) {
        let mut solvables = self.solvables.borrow_mut();
        solvables.clear();
        for pkgs in self.registry.values() {
            for pkg in pkgs {
                solvables.push(pkg.clone());
            }
        }
    }

    fn intern_string(&self, s: &str) -> StringId {
        let mut map = self.string_to_id.borrow_mut();
        if let Some(&id) = map.get(s) {
            return StringId(id);
        }
        let mut strings = self.strings.borrow_mut();
        let id = strings.len() as u32;
        strings.push(s.to_string());
        map.insert(s.to_string(), id);
        StringId(id)
    }

    pub fn intern_package_name(&self, name: &str) -> NameId {
        let mut map = self.name_to_id.borrow_mut();
        if let Some(&id) = map.get(name) {
            return id;
        }
        let mut names = self.names.borrow_mut();
        let id = NameId(names.len() as u32);
        names.push(name.to_string());
        map.insert(name.to_string(), id);
        id
    }

    pub fn intern_version_set(&self, pkg_name: NameId, req: VersionReq) -> VersionSetId {
        let mut sets = self.version_sets.borrow_mut();
        let id = VersionSetId(sets.len() as u32);
        sets.push((pkg_name, req));
        id
    }
}

impl Interner for SatEngine {
    fn display_string(&self, string_id: StringId) -> impl Display + '_ {
        self.strings.borrow()[string_id.0 as usize].clone()
    }

    fn display_name(&self, name_id: NameId) -> impl Display + '_ {
        self.names.borrow()[name_id.0 as usize].clone()
    }

    fn display_version_set(&self, version_set_id: VersionSetId) -> impl Display + '_ {
        let idx = version_set_id.0 as usize;
        let sets = self.version_sets.borrow();
        let (name_id, ref req) = sets[idx];
        let name = self.names.borrow()[name_id.0 as usize].clone();
        format!("{} {}", name, req)
    }

    fn display_solvable(&self, solvable_id: SolvableId) -> impl Display + '_ {
        if solvable_id.0 == 0 {
            // Root check: assuming 0 is root
            return "root".to_string();
        }
        let idx = solvable_id.0 as usize - 1;
        match self.solvables.borrow().get(idx) {
            Some(pkg) => format!("{} @ {}", pkg.name, pkg.version),
            None => "unknown".to_string(),
        }
    }

    fn version_set_name(&self, version_set_id: VersionSetId) -> NameId {
        let idx = version_set_id.0 as usize;
        self.version_sets.borrow()[idx].0
    }

    fn solvable_name(&self, solvable_id: SolvableId) -> NameId {
        if solvable_id.0 == 0 {
            return self.intern_package_name("root");
        }
        let idx = solvable_id.0 as usize - 1;
        let solvables = self.solvables.borrow();
        let pkg = &solvables[idx];
        self.intern_package_name(&pkg.name)
    }

    fn version_sets_in_union(
        &self,
        _version_set_union_id: VersionSetUnionId,
    ) -> impl Iterator<Item = VersionSetId> {
        std::iter::empty()
    }

    fn resolve_condition(&self, _condition_id: ConditionId) -> Condition {
        panic!("Conditions not implemented yet");
    }
}

impl DependencyProvider for SatEngine {
    async fn filter_candidates(
        &self,
        candidates: &[SolvableId],
        version_set_id: VersionSetId,
        inverse: bool,
    ) -> Vec<SolvableId> {
        let idx = version_set_id.0 as usize;
        let req = {
            let sets = self.version_sets.borrow();
            sets[idx].1.clone()
        };

        let solvables = self.solvables.borrow();

        candidates
            .iter()
            .copied()
            .filter(|&id| {
                if id.0 == 0 {
                    return false;
                }
                let s_idx = id.0 as usize - 1;
                if let Some(pkg) = solvables.get(s_idx) {
                    let matches = req.matches(&pkg.version);
                    if inverse {
                        !matches
                    } else {
                        matches
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    async fn sort_candidates(&self, _solver: &SolverCache<Self>, candidates: &mut [SolvableId]) {
        let solvables = self.solvables.borrow();
        candidates.sort_by(|&a, &b| {
            if a.0 == 0 {
                return std::cmp::Ordering::Less;
            }
            if b.0 == 0 {
                return std::cmp::Ordering::Greater;
            }

            let pkg_a = &solvables[a.0 as usize - 1];
            let pkg_b = &solvables[b.0 as usize - 1];
            pkg_b.version.cmp(&pkg_a.version)
        });
    }

    async fn get_candidates(&self, name_id: NameId) -> Option<Candidates> {
        let name = {
            let names = self.names.borrow();
            names[name_id.0 as usize].clone()
        };

        let solvables = self.solvables.borrow();
        let ids: Vec<SolvableId> = solvables
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name == name)
            .map(|(i, _)| SolvableId((i + 1) as u32))
            .collect();

        if ids.is_empty() {
            None
        } else {
            Some(Candidates {
                candidates: ids,
                ..Candidates::default()
            })
        }
    }

    async fn get_dependencies(&self, solvable_id: SolvableId) -> Dependencies {
        if solvable_id.0 == 0 {
            return Dependencies::Known(KnownDependencies::default());
        }

        let idx = solvable_id.0 as usize - 1;
        let pkg_deps = {
            let solvables = self.solvables.borrow();
            solvables[idx].deps.clone()
        };

        let mut result = KnownDependencies::default();
        for (dep_name, dep_req) in pkg_deps {
            let name_id = self.intern_package_name(&dep_name);
            let version_set_id = self.intern_version_set(name_id, dep_req);
            // Assuming 0.10, we can push VersionSetId if we convert
            // If .into() exists, easy. Otherwise use constructor.
            // The compiler said Into::into is helps.
            result.requirements.push(version_set_id.into());
        }

        Dependencies::Known(result)
    }
}
