import { DofusWindow } from "../../types";

export type WindowListProps = {
    windows: DofusWindow[];
    activeDofusWindow: DofusWindow | null;
    focusWindow: (hwnd: number) => void;
}