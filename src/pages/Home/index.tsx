// pages/Home/Home.tsx
import { ActionIcon, Box, Flex } from '@mantine/core';
import { IconAdjustments, IconRefresh, IconSword } from '@tabler/icons-react';
import React, { useRef, useState } from 'react';
import { useAlwaysOntop } from '../../hook/useAlwaysOntop';
import { useShortcuts } from '../../hook/useShortcuts';
import { useWindows } from '../../hook/useWindows';
import { useWindowSizeAdjuster } from '../../hook/useWindowSizeAdjuster';
import { Settings } from './components/Settings';
import { WindowList } from './components/WindowList';

export const Home: React.FC = () => {
  const [selectedTab, setSelectedTab] = useState<'windows' | 'settings'>(
    'windows'
  );

  const { windows, activeDofusWindow, refreshWindows, focusWindow } =
    useWindows();
  const {
    shortcuts,
    watchingInput,
    registerShortcut,
    clickAllDelays,
    removeShortcut,
    updateClickAllDelays,
  } = useShortcuts();
  const { isAlwaysOnTop, toggleAlwaysOnTop } = useAlwaysOntop();

  const contentRef = useRef<HTMLDivElement>(null);

  const { toggleAutoAdjustSize, autoAdjustSize } = useWindowSizeAdjuster(
    contentRef,
    [windows, selectedTab, watchingInput, !!activeDofusWindow]
  );

  return (
    <Box
      id="titlebar"
      ref={contentRef}
      h="100%"
      w="fit-content"
      style={{ overflow: 'hidden' }}
      data-tauri-drag-region
    >
      <Flex
        justify="space-between"
        align="center"
        px="md"
        py="xs"
        w="100%"
        data-tauri-drag-region
        style={{ overflow: 'hidden' }}
        bg="#1e1e1e"
      >
        <ActionIcon
          variant="filled"
          aria-label="Refresh windows"
          color="gray"
          onClick={refreshWindows}
          title="Refresh windows"
        >
          <IconRefresh style={{ width: '70%', height: '70%' }} stroke={1.5} />
        </ActionIcon>
        <ActionIcon
          variant="filled"
          aria-label="Toggle settings"
          color="gray"
          onClick={() =>
            setSelectedTab((prev) =>
              prev === 'windows' ? 'settings' : 'windows'
            )
          }
          title={selectedTab === 'windows' ? 'Open settings' : 'Show windows'}
        >
          {selectedTab === 'windows' ? (
            <IconAdjustments
              style={{ width: '60%', height: '60%' }}
              stroke={1.5}
            />
          ) : (
            <IconSword style={{ width: '70%', height: '70%' }} stroke={1.5} />
          )}
        </ActionIcon>
      </Flex>

      {selectedTab === 'windows' && (
        <WindowList
          windows={windows}
          activeDofusWindow={activeDofusWindow}
          focusWindow={focusWindow}
        />
      )}

      {selectedTab === 'settings' && (
        <Settings
          autoAdjustSize={autoAdjustSize}
          toggleAutoAdjustSize={toggleAutoAdjustSize}
          isAlwaysOnTop={isAlwaysOnTop}
          toggleAlwaysOnTop={toggleAlwaysOnTop}
          shortcuts={shortcuts}
          watchingInput={watchingInput}
          registerShortcut={registerShortcut}
          clickAllDelays={clickAllDelays}
          updateClickAllDelays={updateClickAllDelays}
          removeShortcut={removeShortcut}
        />
      )}
    </Box>
  );
};
