import { ActionIcon, Flex, Kbd, Loader, Paper, Text } from '@mantine/core';
import { IconReplace, IconTrash } from '@tabler/icons-react';
import { ShortcutSettingProps } from './types';

const formatShortcut = (shortcut: string) => {
  if (shortcut.includes('Unknown')) {
    return shortcut.replace('Unknown', 'Mouse');
  }
  return shortcut;
};

export const ShortcutSettingInput: React.FC<ShortcutSettingProps> = ({
  label,
  shortcutType,
  shortcutKey,
  onRegister,
  onRemove,
  watching,
}) => (
  <Paper shadow="md" radius="md" w="100%" bg={'#2e2e2e'} p="md">
    <Flex direction="column" gap="xs">
      <Text fw={500}>{label} :</Text>
      <Flex align="center" gap="sm">
        <Kbd>
          {shortcutKey ? formatShortcut(shortcutKey) : 'No Shortcut set'}
        </Kbd>
        {!watching ? (
          <ActionIcon
            variant="filled"
            aria-label={`Set shortcut for ${label}`}
            onClick={() => onRegister(shortcutType)}
            color="gray"
          >
            <IconReplace style={{ width: '70%', height: '70%' }} stroke={1.5} />
          </ActionIcon>
        ) : (
          <Loader />
        )}
        <ActionIcon
          variant="filled"
          color="red"
          disabled={watching}
          aria-label={`Remove shortcut for ${label}`}
          onClick={() => onRemove(shortcutType)}
        >
          <IconTrash style={{ width: '70%', height: '70%' }} stroke={1.5} />
        </ActionIcon>
      </Flex>
    </Flex>
  </Paper>
);
