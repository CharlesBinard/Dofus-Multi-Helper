import { Flex, Text } from '@mantine/core';
import React from 'react';
import { WindowItem } from '../../../../components/WindowItem';
import { WindowListProps } from './types';

export const WindowList: React.FC<WindowListProps> = ({
  windows,
  activeDofusWindow,
  focusWindow,
}) => {
  return (
    <Flex
      gap="sm"
      direction="column"
      p="sm"
      style={{
        width: 'max-content',
        minWidth: 'min-content',
      }}
    >
      {windows.length === 0 && <Text c="dimmed">No Dofus windows found.</Text>}
      {windows.map((win) => (
        <WindowItem
          key={win.hwnd}
          window={win}
          isActive={activeDofusWindow?.hwnd === win.hwnd}
          focusWindow={focusWindow}
        />
      ))}
    </Flex>
  );
};
