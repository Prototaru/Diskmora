import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FileTree } from "./FileTree";
import { FileTypeFilters } from "./FileTypeFilters";
import { useScanStore } from "./scanStore";
import { type FileTypeOption } from "./types";

export function ScanPage() {
    const scan = useScanStore();
    const [types, setTypes] = useState<FileTypeOption[]>([]);
    const [error, setError] = useState("");
    useEffect(() => {
        invoke<FileTypeOption[]>("file_types").then(setTypes).catch((value) => setError(String(value)));
    }, []);

    async function browse() {
        try {
            const path = await open({ directory: true, multiple: false, title: "Choose a folder to scan" });
            if (typeof path === "string") scan.setPath(path);
        } catch (value) {
            setError(String(value));
        }
    }

    return <main>
        <section className="workspace">
            <section className="scan-setup" aria-label="Scan setup">
                <div className="scan-toolbar"><label htmlFor="folder">Target folder</label>
                    <div className="folder"><input id="folder" placeholder="/Volumes/Archive" value={scan.path} disabled={scan.running} onChange={(event) => scan.setPath(event.target.value)} />
                        <button disabled={scan.running} onClick={browse}>Browse</button>
                    </div>
                    {scan.running ? <button className="primary stop" onClick={scan.cancel}>Stop scan</button> :
                        <button className="primary" disabled={!scan.path.trim() || !scan.selected.length || !types.length} onClick={() => void scan.start()}>Start scan <span>↗</span></button>}
                </div>
                <FileTypeFilters types={types} />
            </section>
            <FileTree key={scan.resultId} files={scan.progress.files} types={types} progress={scan.progress} status={scan.status} running={scan.running} error={error || scan.error} clear={() => {
                setError("");
                scan.clear();
            }} />
        </section>
        <footer><span>DISKMORA / 0.1</span><span>File signatures, not filenames.</span></footer>
    </main>;
}
