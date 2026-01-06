"use client";

import { useEffect, useState } from "react";
import { Copy, CheckCircle, AlertTriangle, XCircle, RefreshCw } from "lucide-react";

interface ScanResult {
    packge_name: String;
    version: String;
    status: String;
    score: number;
    report: any;
    created_at: String;
}

export default function DashboardPage() {
    const [scans, setScans] = useState<ScanResult[]>([]);
    const [loading, setLoading] = useState(true);

    const fetchScans = async () => {
        setLoading(true);
        try {
            const res = await fetch("http://localhost:3000/api/scans/stats");
            if (res.ok) {
                const data = await res.json();
                setScans(data);
            }
        } catch (e) {
            console.error("Failed to fetch scans", e);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchScans();
        const interval = setInterval(fetchScans, 5000); // Poll every 5s
        return () => clearInterval(interval);
    }, []);

    return (
        <div className="min-h-screen bg-neutral-900 text-white p-8 font-sans">
            <header className="mb-8 flex justify-between items-center">
                <div>
                    <h1 className="text-3xl font-bold bg-gradient-to-r from-blue-400 to-emerald-400 bg-clip-text text-transparent">
                        Notary Access Network
                    </h1>
                    <p className="text-neutral-400 mt-2">Worker Status & Real-time Scan Feed</p>
                </div>
                <div className="flex items-center gap-4">
                    <div className="flex items-center gap-2">
                        <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                        <span className="text-sm text-emerald-500 font-mono">WORKER ONLINE</span>
                    </div>
                    <button
                        onClick={fetchScans}
                        className="p-2 bg-neutral-800 rounded-lg hover:bg-neutral-700 transition-colors"
                    >
                        <RefreshCw className={`w-5 h-5 ${loading ? "animate-spin" : ""}`} />
                    </button>
                </div>
            </header>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                {/* Stats Cards */}
                <div className="lg:col-span-3 grid grid-cols-1 md:grid-cols-3 gap-4">
                    <StatCard title="Scans Today" value={scans.length.toString()} color="blue" />
                    <StatCard title="Malicious Blocked" value={scans.filter(s => s.status.includes("Malicious")).length.toString()} color="red" />
                    <StatCard title="Avg Trust Score" value={`${Math.round(scans.reduce((acc, s) => acc + s.score, 0) / (scans.length || 1))}%`} color="emerald" />
                </div>

                {/* Live Feed */}
                <div className="lg:col-span-3 bg-neutral-950 border border-neutral-800 rounded-xl overflow-hidden shadow-2xl">
                    <div className="p-4 border-b border-neutral-800 bg-neutral-900/50 backdrop-blur">
                        <h2 className="text-lg font-semibold">Live Verification Stream</h2>
                    </div>
                    <div className="overflow-x-auto">
                        <table className="w-full text-left text-sm">
                            <thead className="bg-neutral-900 text-neutral-400 uppercase font-mono text-xs">
                                <tr>
                                    <th className="p-4">Package</th>
                                    <th className="p-4">Version</th>
                                    <th className="p-4">Status</th>
                                    <th className="p-4">Score</th>
                                    <th className="p-4">Findings</th>
                                    <th className="p-4">Timestamp</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-neutral-800">
                                {scans.length === 0 ? (
                                    <tr>
                                        <td colSpan={6} className="p-8 text-center text-neutral-500">
                                            Waiting for incoming artifacts...
                                        </td>
                                    </tr>
                                ) : (
                                    scans.map((scan, idx) => (
                                        <tr key={idx} className="hover:bg-neutral-800/30 transition-colors group">
                                            <td className="p-4 font-mono text-blue-300">{scan.packge_name}</td>
                                            <td className="p-4 text-neutral-300">{scan.version}</td>
                                            <td className="p-4">
                                                <StatusBadge status={scan.status} />
                                            </td>
                                            <td className="p-4">
                                                <ScoreBar score={scan.score} />
                                            </td>
                                            <td className="p-4 text-neutral-400 max-w-xs truncate">
                                                {scan.report.suspicious_strings?.length > 0 && (
                                                    <span className="text-red-400 flex items-center gap-1">
                                                        <AlertTriangle className="w-3 h-3" />
                                                        {scan.report.suspicious_strings[0]}
                                                    </span>
                                                )}
                                                {scan.report.suspicious_strings?.length === 0 && (
                                                    <span className="text-emerald-500/50 flex items-center gap-1">
                                                        <CheckCircle className="w-3 h-3" />
                                                        Clean
                                                    </span>
                                                )}
                                            </td>
                                            <td className="p-4 text-neutral-500 font-mono text-xs">
                                                {new Date(scan.created_at.toString()).toLocaleString()}
                                            </td>
                                        </tr>
                                    ))
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
}

function StatusBadge({ status }: { status: String }) {
    if (status.includes("Safe")) {
        return <span className="px-2 py-1 rounded-full bg-emerald-500/10 text-emerald-400 text-xs border border-emerald-500/20 font-medium">SAFE</span>;
    }
    if (status.includes("Malicious")) {
        return <span className="px-2 py-1 rounded-full bg-red-500/10 text-red-400 text-xs border border-red-500/20 font-medium animate-pulse">MALICIOUS</span>;
    }
    return <span className="px-2 py-1 rounded-full bg-yellow-500/10 text-yellow-400 text-xs border border-yellow-500/20 font-medium">SUSPICIOUS</span>;
}

function ScoreBar({ score }: { score: number }) {
    let color = "bg-red-500";
    if (score > 80) color = "bg-emerald-500";
    else if (score > 50) color = "bg-yellow-500";

    return (
        <div className="flex items-center gap-2">
            <div className="w-24 h-2 bg-neutral-800 rounded-full overflow-hidden">
                <div className={`h-full ${color}`} style={{ width: `${score}%` }}></div>
            </div>
            <span className="text-xs font-mono">{score}</span>
        </div>
    );
}

function StatCard({ title, value, color }: { title: string, value: string, color: string }) {
    const colors: Record<string, string> = {
        blue: "from-blue-500/20 to-blue-500/5 border-blue-500/20 text-blue-400",
        red: "from-red-500/20 to-red-500/5 border-red-500/20 text-red-400",
        emerald: "from-emerald-500/20 to-emerald-500/5 border-emerald-500/20 text-emerald-400",
    };

    return (
        <div className={`p-6 rounded-xl bg-gradient-to-br ${colors[color]} border backdrop-blur-sm`}>
            <h3 className="text-sm font-medium text-neutral-400 uppercase tracking-wider">{title}</h3>
            <p className={`text-3xl font-bold mt-2 font-mono ${color === 'red' && value !== '0' ? 'text-red-500' : 'text-white'}`}>{value}</p>
        </div>
    );
}
