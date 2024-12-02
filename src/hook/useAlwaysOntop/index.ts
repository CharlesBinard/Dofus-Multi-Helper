import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";

export const useAlwaysOntop = () => {
    const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(localStorage.getItem('alwaysOnTop') === 'true');

    useEffect(() => {
        setAlwaysOnTop(isAlwaysOnTop);
    }, [isAlwaysOnTop]);

    const setAlwaysOnTop = useCallback(async (newState: boolean) => {
        try {
            await invoke('set_tauri_always_on_top', { alwaysOnTop: newState });
            localStorage.setItem('alwaysOnTop', newState.toString());
        } catch (error) {
            console.error('Error setting always on top:', error);
        }
    }, []);

    const toggleAlwaysOnTop = useCallback(async () => {
        setIsAlwaysOnTop(!isAlwaysOnTop);
    }, [isAlwaysOnTop]);

    return { isAlwaysOnTop, toggleAlwaysOnTop };
};