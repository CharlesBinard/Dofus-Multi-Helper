import { DofusWindow } from "../../types";

export type WindowItemProps = {
    window: DofusWindow;
    isActive: boolean;
    focusWindow: (hwnd: number) => void;
}