'use client';

import { useState, useEffect, Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import { Github, Loader2, CheckCircle2, AlertCircle, LogOut } from 'lucide-react';

function PortalContent() {
    const searchParams = useSearchParams();
    const [loading, setLoading] = useState(true);
    const [currentUser, setCurrentUser] = useState<any>(null);
    const [status, setStatus] = useState<{ type: 'success' | 'error' | null; message: string }>({
        type: null,
        message: '',
    });
    const [authorizing, setAuthorizing] = useState(false);

    const userCode = searchParams.get('user_code');
    const API_BASE = 'http://localhost:3000'; // Hardcoded for local dev as requested

    const checkSession = async () => {
        try {
            const res = await fetch(`${API_BASE}/auth/session`, {
                credentials: 'include', // Important for cookie-based sessions
            });
            const data = await res.json();
            if (data.status === 'authenticated') {
                setCurrentUser(data.user);
            } else {
                setCurrentUser(null);
            }
        } catch (e) {
            console.error('Session check failed', e);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        checkSession();
    }, []);

    const handleLogin = () => {
        window.location.href = `${API_BASE}/auth/login`;
    };

    const handleLogout = async () => {
        await fetch(`${API_BASE}/auth/logout`, { credentials: 'include' });
        setCurrentUser(null);
        window.location.reload();
    };

    const handleAuthorize = async () => {
        if (!userCode) return;
        setAuthorizing(true);
        setStatus({ type: null, message: '' });

        try {
            const res = await fetch(`${API_BASE}/auth/verify?user_code=${userCode}`, {
                credentials: 'include',
            });
            const data = await res.json();

            if (res.ok && data.status === 'success') {
                setStatus({
                    type: 'success',
                    message: '✨ CLI Authorized Successfully! You can now close this tab.',
                });
            } else {
                setStatus({
                    type: 'error',
                    message: data.message || 'Authorization failed. Please try again.',
                });
            }
        } catch (e) {
            setStatus({
                type: 'error',
                message: 'Network error. Please ensure the backend is running.',
            });
        } finally {
            setAuthorizing(false);
        }
    };

    if (loading) {
        return (
            <div className="flex flex-col items-center justify-center space-y-4">
                <Loader2 className="w-10 h-10 text-indigo-500 animate-spin" />
                <p className="text-slate-400 font-medium tracking-wide">Synchronizing session...</p>
            </div>
        );
    }

    if (!currentUser) {
        return (
            <div className="flex flex-col items-center text-center space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-700">
                <div>
                    <h1 className="text-4xl font-bold tracking-tight text-white mb-2 font-outfit">Unlock the CLI</h1>
                    <p className="text-slate-400 max-w-sm">Sign in with GitHub to authorize your session and access your developer environment.</p>
                </div>
                <button
                    onClick={handleLogin}
                    className="w-full flex items-center justify-center gap-3 px-6 py-4 bg-[#24292f] hover:bg-[#1b1f23] text-white rounded-2xl font-semibold transition-all hover:-translate-y-1 shadow-xl"
                >
                    <Github className="w-6 h-6" />
                    Continue with GitHub
                </button>
            </div>
        );
    }

    return (
        <div className="flex flex-col items-center text-center space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-700">
            <div>
                <h1 className="text-4xl font-bold tracking-tight text-white mb-2 font-outfit">Confirm Device</h1>
                <p className="text-slate-400 max-w-sm">Link your CLI session with your GitHub identity.</p>
            </div>

            <div className="flex items-center gap-3 px-4 py-2 bg-slate-800/40 border border-slate-700/50 rounded-full shadow-inner">
                <div className="w-8 h-8 rounded-full bg-indigo-500 flex items-center justify-center font-bold text-sm text-white shadow-lg">
                    {currentUser.username.charAt(0).toUpperCase()}
                </div>
                <span className="text-sm font-semibold text-slate-200">{currentUser.username}</span>
            </div>

            <div className="w-full space-y-4">
                <label className="text-xs font-bold text-indigo-400 uppercase tracking-widest">Verification Code</label>
                <div className="relative group">
                    <input
                        type="text"
                        value={userCode || ''}
                        onChange={(e) => {
                            const val = e.target.value.toUpperCase();
                            const params = new URLSearchParams(window.location.search);
                            params.set('user_code', val);
                            window.history.replaceState(null, '', `?${params.toString()}`);
                        }}
                        placeholder="XXXX-XXXX"
                        className="w-full py-6 bg-slate-900/60 border-2 border-dashed border-indigo-500/50 rounded-3xl text-4xl font-black tracking-[0.3em] text-center text-indigo-400 focus:outline-none focus:border-indigo-400 focus:bg-slate-900/80 transition-all placeholder:text-slate-700"
                    />
                </div>
            </div>

            <button
                onClick={handleAuthorize}
                disabled={authorizing || !userCode}
                className="w-full flex items-center justify-center gap-3 px-6 py-4 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-2xl font-bold transition-all hover:-translate-y-1 shadow-lg shadow-indigo-500/20 active:scale-95"
            >
                {authorizing ? <Loader2 className="w-5 h-5 animate-spin" /> : null}
                {status.type === 'success' ? 'Authorized!' : 'Approve & Authorize'}
            </button>

            {status.type && (
                <div className={`w-full flex items-start gap-3 p-4 rounded-xl border text-sm text-left animate-in zoom-in-95 duration-300 ${status.type === 'success'
                    ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400'
                    : 'bg-rose-500/10 border-rose-500/20 text-rose-400'
                    }`}>
                    {status.type === 'success' ? <CheckCircle2 className="w-5 h-5 shrink-0" /> : <AlertCircle className="w-5 h-5 shrink-0" />}
                    <p>{status.message}</p>
                </div>
            )}

            {!status.type && (
                <button
                    onClick={handleLogout}
                    className="text-xs text-slate-500 hover:text-rose-400 transition-colors flex items-center gap-1 group"
                >
                    <LogOut className="w-3 h-3 group-hover:translate-x-0.5 transition-transform" />
                    Not you? Switch account
                </button>
            )}
        </div>
    );
}

export default function PortalPage() {
    return (
        <main className="min-h-screen bg-[#0f172a] relative overflow-hidden flex items-center justify-center p-6 selection:bg-indigo-500/30">
            {/* Dynamic Background */}
            <div className="absolute top-0 left-0 w-full h-full">
                <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] rounded-full bg-indigo-500/10 blur-[120px]" />
                <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] rounded-full bg-purple-500/10 blur-[120px]" />
            </div>

            <div className="w-full max-w-md relative z-10">
                <div className="text-center mb-8">
                    <div className="inline-block px-4 py-1.5 rounded-full bg-slate-800/50 border border-slate-700/50 backdrop-blur-md mb-6">
                        <span className="text-[10px] uppercase tracking-[0.2em] font-bold text-indigo-400">Registry Gateway</span>
                    </div>
                    <div className="text-5xl font-black tracking-tight bg-gradient-to-br from-indigo-300 via-indigo-400 to-purple-400 bg-clip-text text-transparent font-outfit">
                        EnvArchitect
                    </div>
                </div>

                <div className="bg-slate-900/40 backdrop-blur-2xl border border-white/5 rounded-[40px] p-8 shadow-2xl shadow-black/50 relative overflow-hidden group">
                    <div className="absolute inset-0 bg-gradient-to-b from-white/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />
                    <Suspense fallback={
                        <div className="flex flex-col items-center justify-center space-y-4">
                            <Loader2 className="w-10 h-10 text-indigo-500 animate-spin" />
                            <p className="text-slate-400 font-medium tracking-wide">Loading portal...</p>
                        </div>
                    }>
                        <PortalContent />
                    </Suspense>
                </div>
            </div>
        </main>
    );
}
