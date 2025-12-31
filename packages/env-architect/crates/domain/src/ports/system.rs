pub trait SystemInfo {
    fn os_name(&self) -> String;
    fn arch(&self) -> String;
}
