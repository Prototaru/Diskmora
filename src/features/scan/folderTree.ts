import type { ScanFile } from "./types";

export interface FolderNode {
    name: string;
    path: string;
    revealPath: string;
    folders: Map<string, FolderNode>;
    files: ScanFile[];
    count: number;
    size: number;
}

const normalizePath = (path: string) => path.replace(/\\/g, "/").replace(/^\/\/\?\/UNC\//i, "//").replace(/^\/\/\?\//, "");
const parts = (path: string) => normalizePath(path).split("/");
export const fileName = (path: string) => parts(path).at(-1) || path;

export function buildFileTree(files: ScanFile[]): FolderNode {
    const prefix = parts(files[0]?.path ?? "").slice(0, -1);
    for (const file of files) {
        const directory = parts(file.path).slice(0, -1);
        let length = 0;
        while (length < prefix.length && prefix[length] === directory[length]) length++;
        prefix.length = length;
    }
    const commonPath = prefix.join("/") || "/";
    const path = /^[a-z]:$/i.test(commonPath) ? `${commonPath}/` : commonPath;
    const root: FolderNode = { name: path, path, revealPath: path, folders: new Map(), files: [], count: 0, size: 0 };
    for (const file of files) {
        let node = root;
        const archivePath = file.archivePath ? normalizePath(file.archivePath) : undefined;
        if (archivePath && node.path.startsWith(`${archivePath}!`)) node.revealPath = file.archivePath!;
        node.count++;
        node.size += file.size;
        for (const name of parts(file.path).slice(prefix.length, -1)) {
            const childPath = `${node.path.replace(/\/$/, "")}/${name}`;
            if (!node.folders.has(name)) node.folders.set(name, {
                name, path: childPath, revealPath: archivePath && childPath.startsWith(`${archivePath}!`) ? file.archivePath! : childPath, folders: new Map(), files: [], count: 0, size: 0
            });
            node = node.folders.get(name)!;
            node.count++;
            node.size += file.size;
        }
        node.files.push(file);
    }
    return root;
}

export interface SortState {
    field: "name" | "size" | "type";
    descending: boolean;
}

export function compareTreeItems(aName: string, aSize: number, aType: string | number, bName: string, bSize: number, bType: string | number, sort: SortState): number {
    const names = aName.localeCompare(bName, undefined, { numeric: true, sensitivity: "base" });
    const comparison = sort.field === "size" ? aSize - bSize : sort.field === "type"
        ? typeof aType === "number" && typeof bType === "number" ? aType - bType : String(aType).localeCompare(String(bType)) : names;
    return (comparison || names) * (sort.descending ? -1 : 1);
}
