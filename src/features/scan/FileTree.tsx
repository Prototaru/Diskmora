import { useDeferredValue, useMemo, useState } from "react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { buildFileTree, compareTreeItems, fileName, type FolderNode, type SortState } from "./folderTree";
import { SignaturePicker } from "./FileTypeFilters";
import { formatBytes, type FileTypeOption, type ScanFile, type ScanProgress } from "./types";

const revealLabel = /Windows/i.test(navigator.userAgent) ? "Show in File Explorer" : /Macintosh|Mac OS X/i.test(navigator.userAgent) ? "Reveal in Finder" : "Show in File Manager";

type BrowserProps = { files: ScanFile[]; types: FileTypeOption[]; progress: ScanProgress; status: string; running: boolean; error: string; clear: () => void };

export function FileTree({ files, types, progress, status, running, error, clear }: BrowserProps) {
    const [sidebarOpen, setSidebarOpen] = useState(false);
    const [query, setQuery] = useState("");
    const [excluded, setExcluded] = useState<number[]>([]);
    const [revealError, setRevealError] = useState("");
    const [sort, setSort] = useState<SortState>({ field: "name", descending: false });
    const search = useDeferredValue(query.trim().toLocaleLowerCase());
    const filtered = useMemo(() => files.filter((file) => !excluded.includes(types.find((type) => type.label === file.format)?.code ?? -1) && file.path.toLocaleLowerCase().includes(search)), [files, types, excluded, search]);
    const root = useMemo(() => buildFileTree(filtered), [filtered]);
    const options = types.filter((type) => files.some((file) => file.format === type.label));

    async function reveal(path: string) {
        try {
            await revealItemInDir(path);
            setRevealError("");
        } catch (value) {
            setRevealError(String(value));
        }
    }

    return <section className={`result-layout${sidebarOpen ? " sidebar-open" : ""}`} aria-label="Scan results">
        <button className="sidebar-toggle" aria-expanded={sidebarOpen} onClick={() => setSidebarOpen(!sidebarOpen)}>{sidebarOpen ? "Close results panel" : "Results & filters"}</button>
        <section className="tree-area" aria-label="Files grouped by folder">
            <div className="tree-columns" role="row">{([["name", "Name"], ["type", "Items / Type"], ["size", "Size"]] as const).map(([field, label]) => <span role="columnheader" key={field}
                                                                                                                                                              aria-sort={sort.field === field ? sort.descending ? "descending" : "ascending" : "none"}>
        <button onClick={() => setSort({ field, descending: sort.field === field && !sort.descending })}>{label} {sort.field === field ? sort.descending ? "↓" : "↑" : "↕"}</button>
      </span>)}<span>Reveal</span></div>
            <div className="file-tree">{filtered.length ? <Folder node={root} root reveal={reveal} sort={sort} /> :
                <div className="empty"><span>⌕</span><h3>{running ? "Reading file signatures…" : files.length ? "No matching files" : "Ready to scan"}</h3>
                    <p>{files.length ? "Adjust the filters in the sidebar." : "Choose a folder and start scanning."}</p></div>}</div>
        </section>
        <aside className="result-sidebar" aria-label="Result controls">
            <div className="result-heading"><h2>Results</h2>
                <button disabled={running || (!progress.scanned && !progress.skipped && !error && !revealError)} onClick={clear}>Clear</button>
            </div>
            <span role="status" className={running ? "status scanning" : "status"}>{status}</span>
            <div className="stats">
                <div><strong>{progress.scanned.toLocaleString()}</strong><span>FILES CHECKED</span></div>
                <div><strong>{progress.matched.toLocaleString()}</strong><span>MATCHES</span></div>
                <div><strong>{formatBytes(progress.bytes)}</strong><span>MATCHED SIZE</span></div>
            </div>
            <label className="sidebar-label" htmlFor="result-search">Search results</label><input id="result-search" placeholder="File or folder path…" value={query}
                                                                                                  onChange={(event) => setQuery(event.target.value)} />
            <div className="sidebar-actions">
                <button disabled={!query && !excluded.length} onClick={() => {
                    setQuery("");
                    setExcluded([]);
                }}>Reset filters
                </button>
            </div>
            <SignaturePicker label="Result formats" types={options} selected={options.filter((type) => !excluded.includes(type.code)).map((type) => type.code)}
                             toggle={(code) => setExcluded((values) => values.includes(code) ? values.filter((value) => value !== code) : [...values, code])}
                             toggleGroup={(codes) => setExcluded((values) => codes.every((code) => !values.includes(code)) ? [...new Set([...values, ...codes])] : values.filter((code) => !codes.includes(code)))} />
            <p className="filter-summary">{filtered.length.toLocaleString()} of {files.length.toLocaleString()} loaded files · {formatBytes(root.size)}</p>
            <p className="hint">Filters and folder totals apply to loaded results. Archive entry sizes are uncompressed; totals include containers and entries.</p>
            {progress.matched > 2000 ? <p className="hint">Only the first 2,000 matches are loaded.</p> : null}
            <p className="current" title={progress.currentPath}>{progress.currentPath || "No active file"}</p>
            {error || revealError ? <p className="error" role="alert">{error || revealError}</p> : null}
            {progress.unreadDirectories.length ? <details className="unread-directories">
                <summary>{progress.unreadDirectories.length} unreadable directories skipped</summary>
                <ul>{progress.unreadDirectories.map((path) => <li key={path}>{path}</li>)}</ul>
            </details> : null}
            {progress.skipped ? <details>
                <summary>{progress.skipped} entries could not be read</summary>
                {progress.errors.map((value, index) => <p className="error" key={index}>{value}</p>)}<p className="hint">Showing up to 20 errors.</p></details> : null}
        </aside>
    </section>;
}

function Folder({ node, root = false, reveal, sort }: { node: FolderNode; root?: boolean; reveal: (path: string) => Promise<void>; sort: SortState }) {
    const [expanded, setExpanded] = useState(root);
    return <div className="tree-folder">
        <div className="tree-row folder-row">
            <button className="tree-folder-toggle" aria-expanded={expanded} title={node.path} onClick={() => setExpanded(!expanded)}><span className="tree-chevron"
                                                                                                                                           aria-hidden="true">{expanded ? "▾" : "▸"}</span>
                <svg className="folder-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true">
                    <path d="M3 7V5h6l2 2h10v12H3Z" />
                </svg>
                <span className="tree-name">{root ? node.path : node.name.replace(/!$/, " (archive)")}</span></button>
            <span className="tree-count">{node.count} files</span><span className="tree-size">{formatBytes(node.size)}</span>
            <button className="reveal" title={revealLabel} aria-label={`${revealLabel}: ${node.name}`} onClick={() => void reveal(node.revealPath)}>↗</button>
        </div>
        {expanded ? <div className="tree-children">
            {[...node.folders.values()].sort((a, b) => compareTreeItems(a.name, a.size, a.count, b.name, b.size, b.count, sort)).map((folder) => <Folder key={folder.path} node={folder} reveal={reveal}
                                                                                                                                                         sort={sort} />)}
            {[...node.files].sort((a, b) => compareTreeItems(fileName(a.path), a.size, a.format, fileName(b.path), b.size, b.format, sort)).map((file) => <div className="tree-row tree-file"
                                                                                                                                                               key={file.path}>
                <span className="tree-name" title={file.path}><span className="file-symbol" aria-hidden="true">▪</span>{fileName(file.path)}</span><span className="format">{file.format}</span><span
                className="tree-size">{formatBytes(file.size)}</span>
                <button className="reveal" title={revealLabel} aria-label={`${revealLabel}: ${fileName(file.path)}`} onClick={() => void reveal(file.archivePath || file.path)}>↗</button>
            </div>)}
        </div> : null}
    </div>;
}
