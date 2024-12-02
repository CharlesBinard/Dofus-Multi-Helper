import { Flex, Image, Paper, Text } from '@mantine/core';
import React from 'react';
import { CLASSES_IMAGES } from '../../assets/classes/constants';
import { WindowItemProps } from './types';

export const WindowItem: React.FC<WindowItemProps> = ({
  window: win,
  isActive,
  focusWindow,
}) => {
  return (
    <Paper
      shadow="md"
      radius="md"
      pr="xs"
      key={win.hwnd}
      withBorder={isActive}
      bd={isActive ? '2px dashed gray.5' : '1px solid gray.7'}
      bg={isActive ? '#1e1e1e' : '#2e2e2e'}
      onClick={() => focusWindow(win.hwnd)}
      style={{ width: '100%', cursor: 'pointer' }}
    >
      <Flex direction="row" align="center" gap="xs">
        <Image
          height={40}
          width={40}
          src={
            CLASSES_IMAGES[
              win.class.toLowerCase() as keyof typeof CLASSES_IMAGES
            ] || CLASSES_IMAGES['unknown']
          }
          alt={win.class || 'unknown'}
          style={{ flexShrink: 0 }}
          radius="md"
        />
        <Text
          fw={500}
          size="lg"
          style={{
            whiteSpace: 'nowrap',
          }}
        >
          {win.name}
        </Text>
      </Flex>
    </Paper>
  );
};
