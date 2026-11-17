export interface FileTypeOption {
    code: number;
    label: string;
}

export interface ScanFile {
    path: string;
    format: string;
    size: number;
    archivePath?: string | null;
    entryPath?: string | null;
}

export interface ScanProgress {
    scanned: number;
    matched: number;
    bytes: number;
    skipped: number;
    permissionDenied: number;
    currentPath: string;
    files: ScanFile[];
    errors: string[];
    unreadDirectories: string[];
    cancelled: boolean;
}

export const emptyProgress: ScanProgress = {
    scanned: 0, matched: 0, bytes: 0, skipped: 0, permissionDenied: 0,
    currentPath: "", files: [], errors: [], unreadDirectories: [], cancelled: false
};

export function formatBytes(bytes: number): string {
    if (!bytes) return "0 B";
    const unit = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), 4);
    return `${(bytes / 1024 ** unit).toFixed(unit ? 1 : 0)} ${["B", "KB", "MB", "GB", "TB"][unit]}`;
}
