import { invoke } from '@tauri-apps/api/core';
import { UnlistenFn, listen } from '@tauri-apps/api/event';
import { useCallback, useEffect, useState } from 'react';
import { Shortcuts } from '../../types';
import { ClickAllDelays, UpdateClickAllDelaysParams } from './types';

export const useShortcuts = () => {
    const [shortcuts, setShortcuts] = useState<Shortcuts>({
        next: '',
        prev: '',
        click_all: '',
        click_all_delay: '',
    });
    const [watchingInput, setWatchingInput] = useState<keyof Shortcuts | undefined>(undefined);

    const storageClickAllDelays = JSON.parse(localStorage.getItem('clickAllDelays') || '{ "min": 100, "max": 130 }')
    const [clickAllDelays, setClickAllDelays] = useState<ClickAllDelays>(storageClickAllDelays);

    const updateClickAllDelays = ({ min, max }: UpdateClickAllDelaysParams) => {
        setClickAllDelays((prev) => {
            const newDelays = {
                ...prev,
                min: min ?? prev.min,
                max: max ?? prev.max,
            };
            localStorage.setItem('clickAllDelays', JSON.stringify(newDelays));
            return newDelays;
        });
    };

    const loadShortcuts = useCallback(async () => {
        try {
            const savedShortcuts = await invoke<Shortcuts>('get_shortcuts');
            setShortcuts((prev) => ({
                ...prev,
                ...savedShortcuts,
            }));
        } catch (error) {
            console.error('Failed to load shortcuts: ', error);
        }
    }, []);

    const registerShortcut = useCallback(async (shortcutType: keyof Shortcuts) => {
        try {
            await invoke('register_shortcut', { shortcutType });
            setWatchingInput(shortcutType);
        } catch (error) {
            console.error(`Failed to register shortcut for ${shortcutType}:`, error);
        }
    }, []);

    const removeShortcut = useCallback(async (shortcutType: keyof Shortcuts) => {
        try {
            await invoke('remove_shortcut_key', { shortcutType });
        } catch (error) {
            console.error(`Failed to remove shortcut for ${shortcutType}:`, error);
        }
    }, []);


    const handleShortcutTriggered = useCallback(async (shortcutType: keyof Shortcuts) => {
        try {
            switch (shortcutType) {
                case 'next':
                    await invoke('next_dofus_window');
                    break;
                case 'prev':
                    await invoke('prev_dofus_window');
                    break;
                case 'click_all':
                    await invoke('click_all_dofus_windows');
                    break;
                case 'click_all_delay':
                    await invoke('click_all_dofus_windows_with_delay', {
                        params: {
                            delay_min_ms: clickAllDelays.min,
                            delay_max_ms: clickAllDelays.max
                        }
                    });
                    break;
                default:
                    console.warn(`Unhandled shortcut type: ${shortcutType}`);
            }
        } catch (error) {
            console.error(`Error handling shortcut ${shortcutType}:`, error);
        }
    }, [clickAllDelays]);


    useEffect(() => {
        loadShortcuts();

        let unlistenRegister: Promise<UnlistenFn>;
        let unlistenTrigger: Promise<UnlistenFn>;

        const setupListeners = async () => {
            unlistenRegister = listen<{ shortcut: keyof Shortcuts; key: string }>(
                'input_register_event',
                (event) => {
                    const { shortcut, key } = event.payload;
                    console.log('input_register_event', shortcut, key);
                    if (shortcut) {
                        setShortcuts((prev) => ({
                            ...prev,
                            [shortcut]: key,
                        }));
                        setWatchingInput(undefined);
                    }
                }
            );

            unlistenTrigger = listen<{ shortcut: keyof Shortcuts }>(
                'shortcut_triggered',
                (event) => {
                    const { shortcut } = event.payload;
                    if (shortcut) {
                        handleShortcutTriggered(shortcut);
                    }
                }
            );



        }

        setupListeners();

        return () => {
            [unlistenRegister, unlistenTrigger].forEach(u =>
                u?.then(f => f()).catch(e => console.error('Unlisten error:', e))
            );
        };
    }, [handleShortcutTriggered, loadShortcuts]);



    return {
        shortcuts,
        watchingInput,
        registerShortcut,
        clickAllDelays,
        updateClickAllDelays,
        removeShortcut
    };
};
