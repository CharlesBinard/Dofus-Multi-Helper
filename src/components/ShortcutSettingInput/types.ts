import { Shortcuts } from "../../types";

export type ShortcutSettingProps = {
    label: string;
    shortcutType: keyof Shortcuts;
    shortcutKey: string;
    onRegister: (type: keyof Shortcuts) => void;
    watching: boolean;
};