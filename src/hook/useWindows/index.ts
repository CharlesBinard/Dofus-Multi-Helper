import { invoke } from '@tauri-apps/api/core';
import { UnlistenFn, listen } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';
import { DofusWindow } from '../../types';

export const useWindows = () => {
    const [windows, setWindows] = useState<DofusWindow[]>([]);
    const [activeDofusWindow, setActiveDofusWindow] = useState<DofusWindow | null>(null);

    const fetchWindows = useCallback(async () => {
        try {
            const result: DofusWindow[] = await invoke('get_dofus_windows');
            setWindows(result);
        } catch (error) {
            console.error('Error fetching windows:', error);
        }
    }, []);

    const focusWindow = useCallback(async (hwnd: number) => {
        try {
            await invoke('focus_window_command', { hwnd });
        } catch (error) {
            console.error('Error focusing window:', error);
        }
    }, []);

    const refreshWindows = useCallback(async () => {
        try {
            await invoke('refresh_windows');
        } catch (error) {
            console.error('Error focusing window:', error);
        }
    }, []);

    useEffect(() => {
        fetchWindows();
        let unlistenActive: Promise<UnlistenFn>;
        let unlistenRefresh: Promise<UnlistenFn>;

        const setupListeners = async () => {
            unlistenActive = listen<DofusWindow | null>(
                'active_dofus_changed',
                (event) => {
                    setActiveDofusWindow(event.payload);
                }
            );

            unlistenRefresh = listen<DofusWindow[]>(
                'dofus_windows_changed',
                (event) => {
                    setWindows(event.payload);
                }
            );
        };

        setupListeners();

        return () => {
            [unlistenActive, unlistenRefresh].forEach(u =>
                u?.then(f => f()).catch(e => console.error('Unlisten error:', e))
            );
        };
    }, [fetchWindows]);


    return {
        windows,
        activeDofusWindow,
        refreshWindows,
        fetchWindows,
        focusWindow,
    };
};
