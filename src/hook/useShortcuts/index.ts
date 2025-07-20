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
    auto_follow_leader: '',
    auto_invite_all: '',
  });
  const [watchingInput, setWatchingInput] = useState<
    keyof Shortcuts | undefined
  >(undefined);
  const [watchingAutoFollowLeaderKey, setWatchingAutoFollowLeaderKey] =
    useState<boolean>(false);

  const storageClickAllDelays = JSON.parse(
    localStorage.getItem('clickAllDelays') || '{ "min": 100, "max": 130 }'
  );
  const [clickAllDelays, setClickAllDelays] = useState<ClickAllDelays>(
    storageClickAllDelays
  );
  const [autoFollowLeaderKey, setAutoFollowLeaderKey] = useState<string>(
    localStorage.getItem('autoFollowLeaderKey') || '/'
  );

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

  const handleSetAutoFollowLeaderKey = (key: string) => {
    setAutoFollowLeaderKey(key);
    localStorage.setItem('autoFollowLeaderKey', key);
  };

  const watchAutoFollowLeaderKey = async () => {
    try {
      await invoke('watch_key_to_send');
      setWatchingAutoFollowLeaderKey(true);
    } catch (error) {
      console.error('Failed to watch key to send:', error);
    }
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

  const registerShortcut = useCallback(
    async (shortcutType: keyof Shortcuts) => {
      try {
        await invoke('register_shortcut', { shortcutType });
        setWatchingInput(shortcutType);
      } catch (error) {
        console.error(
          `Failed to register shortcut for ${shortcutType}:`,
          error
        );
      }
    },
    []
  );

  const removeShortcut = useCallback(async (shortcutType: keyof Shortcuts) => {
    try {
      await invoke('remove_shortcut_key', { shortcutType });
    } catch (error) {
      console.error(`Failed to remove shortcut for ${shortcutType}:`, error);
    }
  }, []);

  const handleShortcutTriggered = useCallback(
    async (shortcutType: keyof Shortcuts) => {
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
                delay_max_ms: clickAllDelays.max,
              },
            });
            break;
          case 'auto_follow_leader':
            await invoke('send_key_to_all_dofus_windows', {
              key: autoFollowLeaderKey,
              repeat: 2,
            });
            break;
          case 'auto_invite_all':
            console.log('🚀 Auto invite all triggered!');
            try {
              await invoke('auto_invite_all_characters');
              console.log('✅ Auto invite all completed successfully');
            } catch (error) {
              console.error('❌ Error in auto_invite_all:', error);
            }
            break;
          default:
            console.warn(`Unhandled shortcut type: ${shortcutType}`);
        }
      } catch (error) {
        console.error(`Error handling shortcut ${shortcutType}:`, error);
      }
    },
    [clickAllDelays, autoFollowLeaderKey]
  );

  useEffect(() => {
    loadShortcuts();

    let unlistenRegister: Promise<UnlistenFn>;
    let unlistenTrigger: Promise<UnlistenFn>;
    let unlistenAutoFollowLeaderKey: Promise<UnlistenFn>;

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
          console.log('🔥 Shortcut triggered event received:', shortcut);
          if (shortcut) {
            handleShortcutTriggered(shortcut);
          }
        }
      );

      unlistenAutoFollowLeaderKey = listen<{ key: string }>(
        'key_to_send_set',
        (event) => {
          const { key } = event.payload;
          handleSetAutoFollowLeaderKey(key);
          setWatchingAutoFollowLeaderKey(false);
        }
      );
    };

    setupListeners();

    return () => {
      [unlistenRegister, unlistenTrigger, unlistenAutoFollowLeaderKey].forEach(
        (u) =>
          u?.then((f) => f()).catch((e) => console.error('Unlisten error:', e))
      );
    };
  }, [handleShortcutTriggered, loadShortcuts]);

  return {
    shortcuts,
    watchingInput,
    registerShortcut,
    clickAllDelays,
    updateClickAllDelays,
    removeShortcut,
    autoFollowLeaderKey,
    setAutoFollowLeaderKey: handleSetAutoFollowLeaderKey,
    watchingAutoFollowLeaderKey,
    watchAutoFollowLeaderKey,
  };
};
