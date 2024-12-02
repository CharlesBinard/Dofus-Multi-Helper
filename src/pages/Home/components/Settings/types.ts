import { ClickAllDelays, UpdateClickAllDelaysParams } from "../../../../hook/useShortcuts/types";
import { Shortcuts } from "../../../../types";

export type SettingsProps = {
    autoAdjustSize: boolean;
    toggleAutoAdjustSize: () => void;
    isAlwaysOnTop: boolean;
    toggleAlwaysOnTop: () => void;
    shortcuts: Shortcuts;
    watchingInput?: string;
    registerShortcut: (type: keyof Shortcuts) => void;
    clickAllDelays: ClickAllDelays;
    updateClickAllDelays: ({ min, max }: UpdateClickAllDelaysParams) => void;
    removeShortcut: (type: keyof Shortcuts) => void;
}