import { useEffect, useRef } from "react";
import { useScanStore } from "./scanStore";
import type { FileTypeOption } from "./types";

const groups = [
    { label: "Images", codes: [1, 2, 3, 10, 11, 16] },
    { label: "Video", codes: [12, 15, 17, 18, 19, 20, 21, 22, 31, 32, 34, 35] },
    { label: "Audio", codes: [7, 13, 14, 23, 24, 25, 26, 27, 28, 29, 30, 33] },
    { label: "Documents", codes: [4] },
    { label: "Archives", codes: [5, 6] },
    { label: "Executables", codes: [8, 9] },
    { label: "Other", codes: [0] }
];

export function FileTypeFilters({ types }: { types: FileTypeOption[] }) {
    const { selected, running, toggle, toggleGroup } = useScanStore();
    const panel = useRef<HTMLDetailsElement>(null);
    useEffect(() => {
        if (running && panel.current) panel.current.open = false;
    }, [running]);
    return <details ref={panel} className="scan-signatures" onKeyDown={(event) => {
        if (event.key === "Escape") event.currentTarget.open = false;
    }}>
        <summary><span>File signatures</span><span className="selection-count">{selected.length === types.length ? "All formats" : `${selected.length} selected`}</span></summary>
        <div className="signature-panel"><SignaturePicker types={types} selected={selected} disabled={running} toggle={toggle} toggleGroup={toggleGroup} />
            <p className="hint">Hidden files included · ZIP-based documents appear as ZIP · Unknown includes unsupported signatures</p>
        </div>
    </details>;
}

export function SignaturePicker({ types, selected, disabled = false, toggle, toggleGroup, label = "File signatures" }: {
    types: FileTypeOption[]; selected: number[]; disabled?: boolean; label?: string;
    toggle: (code: number) => void; toggleGroup: (codes: number[]) => void;
}) {
    return <fieldset disabled={disabled} className="signature-filters">
        <legend>{label} <span>{types.filter((type) => selected.includes(type.code)).length} selected</span></legend>
        <div className="format-groups">{groups.map((group) => {
            const options = types.filter((type) => group.codes.includes(type.code));
            if (!options.length) return null;
            const count = options.filter((type) => selected.includes(type.code)).length;
            return <fieldset className="format-group" key={group.label}>
                <legend><label className="group-toggle"><input type="checkbox"
                                                               checked={count === options.length}
                                                               ref={(input) => {
                                                                   if (input) input.indeterminate = count > 0 && count < options.length;
                                                               }}
                                                               onChange={() => toggleGroup(options.map((type) => type.code))} />
                    {group.label}<span>{count}/{options.length}</span></label></legend>
                <div className="types">{options.map((type) => <label key={type.code} className="type">
                    <input type="checkbox" checked={selected.includes(type.code)} onChange={() => toggle(type.code)} />{type.label}
                </label>)}</div>
            </fieldset>;
        })}</div>
    </fieldset>;
}
