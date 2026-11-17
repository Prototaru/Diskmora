import { create } from "zustand";
import { Channel, invoke } from "@tauri-apps/api/core";
import { emptyProgress, type ScanProgress } from "./types";

interface ScanState {
    path: string;
    selected: number[];
    running: boolean;
    status: string;
    error: string;
    progress: ScanProgress;
    resultId: number;
    clear: () => void;
    setPath: (path: string) => void;
    toggle: (code: number) => void;
    toggleGroup: (codes: number[]) => void;
    start: () => Promise<void>;
    cancel: () => Promise<void>;
}

export const useScanStore = create<ScanState>((set, get) => ({
    path: "", selected: Array.from({ length: 36 }, (_, code) => code),
    running: false, status: "Ready to explore", error: "", progress: emptyProgress, resultId: 0,
    clear: () => {
        if (!get().running) set({ progress: emptyProgress, error: "", status: "Ready to explore", resultId: get().resultId + 1 });
    },
    setPath: (path) => set({ path }),
    toggle: (code) => set(({ selected }) => ({
        selected: selected.includes(code)
            ? selected.filter((value) => value !== code) : [...selected, code]
    })),
    toggleGroup: (codes) => set(({ selected }) => ({
        selected: codes.every((code) => selected.includes(code))
            ? selected.filter((code) => !codes.includes(code)) : [...new Set([...selected, ...codes])]
    })),
    start: async () => {
        const { path, selected, running } = get();
        if (running || !path.trim() || !selected.length) return;
        set({ running: true, error: "", status: "Scanning", progress: emptyProgress, resultId: get().resultId + 1 });
        const resultId = get().resultId;
        const progress = new Channel<ScanProgress>();
        progress.onmessage = (value) => {
            if (get().running && get().resultId === resultId) set({ progress: value });
        };
        try {
            const result = await invoke<ScanProgress>("scan_folder", { path, types: selected, progress });
            set({ progress: result, status: result.cancelled ? "Scan cancelled" : "Scan complete" });
        } catch (error) {
            set({ error: String(error), status: "Scan failed" });
        } finally {
            set({ running: false });
        }
    },
    cancel: async () => {
        try {
            await invoke("cancel_scan");
            set({ status: "Cancelling" });
        } catch (error) {
            set({ error: String(error) });
        }
    }
}));
