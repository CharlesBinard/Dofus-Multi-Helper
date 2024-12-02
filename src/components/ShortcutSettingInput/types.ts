import { Shortcuts } from "../../types";

export type ShortcutSettingProps = {
    label: string;
    watching: boolean;
    shortcutKey: string;
    shortcutType: keyof Shortcuts;
    onRegister: (type: keyof Shortcuts) => void;
    onRemove: (type: keyof Shortcuts) => void;
};