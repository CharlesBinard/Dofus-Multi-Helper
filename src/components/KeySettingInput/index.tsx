// components/KeySettingInput/index.tsx
import { ActionIcon, Box, Button, Flex, Text } from '@mantine/core';
import { IconX } from '@tabler/icons-react';
import React from 'react';
import { KeySettingInputProps } from './types';

export const KeySettingInput: React.FC<KeySettingInputProps> = ({
  label,
  value,
  onSet,
  onClear,
  watching,
}) => {
  return (
    <Flex direction="column" justify="space-between" align="start" w="100%">
      <Text>{label}</Text>
      <Flex align="center" gap="xs">
        <Box
          style={{
            border: '1px solid #444',
            borderRadius: '4px',
            padding: '4px 8px',
            minWidth: '80px',
            textAlign: 'center',
            backgroundColor: watching ? '#444' : 'transparent',
          }}
        >
          <Text>{watching ? '...' : value}</Text>
        </Box>
        <Button onClick={onSet} variant="light" size="xs">
          Set
        </Button>
        <ActionIcon onClick={onClear} size="sm" variant="subtle" color="red">
          <IconX size={16} />
        </ActionIcon>
      </Flex>
    </Flex>
  );
};
