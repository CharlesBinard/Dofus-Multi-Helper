// components/Settings/Settings.tsx
import { Flex, NumberInput, Paper, Switch, Text } from '@mantine/core';
import React from 'react';
import { ShortcutSettingInput } from '../../../../components/ShortcutSettingInput';
import { SettingsProps } from './types';

export const Settings: React.FC<SettingsProps> = ({
  autoAdjustSize,
  toggleAutoAdjustSize,
  isAlwaysOnTop,
  toggleAlwaysOnTop,
  shortcuts,
  watchingInput,
  registerShortcut,
  clickAllDelays,
  updateClickAllDelays,
  removeShortcut,
}) => {
  return (
    <Flex
      direction="column"
      gap="sm"
      p="md"
      align="stretch"
      style={{
        width: 'max-content',
        minWidth: 'min-content',
        overflow: 'hidden',
      }}
    >
      <Text size="xl" fw={500}>
        Settings
      </Text>

      <Paper shadow="md" radius="md" w="100%" bg={'#2e2e2e'} p="md">
        <Flex direction="column" gap="sm">
          <Switch
            label="Auto resize"
            checked={autoAdjustSize}
            onChange={toggleAutoAdjustSize}
          />
          <Switch
            label="Always on top"
            checked={isAlwaysOnTop}
            onChange={toggleAlwaysOnTop}
          />
          <NumberInput
            radius="md"
            label="Delay min (ms)"
            value={clickAllDelays.min}
            onChange={(value) =>
              updateClickAllDelays({ min: Number(value) || 100 })
            }
          />
          <NumberInput
            radius="md"
            label="Delay max (ms)"
            value={clickAllDelays.max}
            onChange={(value) =>
              updateClickAllDelays({ max: Number(value) || 130 })
            }
          />
        </Flex>
      </Paper>

      <Text size="xl" fw={500}>
        Shortcuts
      </Text>

      <Flex direction="column" gap="sm">
        <ShortcutSettingInput
          label="Next"
          shortcutType="next"
          shortcutKey={shortcuts.next}
          onRegister={registerShortcut}
          onRemove={removeShortcut}
          watching={watchingInput === 'next'}
        />
        <ShortcutSettingInput
          label="Prev"
          shortcutType="prev"
          shortcutKey={shortcuts.prev}
          onRegister={registerShortcut}
          onRemove={removeShortcut}
          watching={watchingInput === 'prev'}
        />
        <ShortcutSettingInput
          label="Click all"
          shortcutType="click_all"
          shortcutKey={shortcuts.click_all}
          onRegister={registerShortcut}
          onRemove={removeShortcut}
          watching={watchingInput === 'click_all'}
        />
        <ShortcutSettingInput
          label="Click all Delay"
          shortcutType="click_all_delay"
          shortcutKey={shortcuts.click_all_delay}
          onRegister={registerShortcut}
          onRemove={removeShortcut}
          watching={watchingInput === 'click_all_delay'}
        />
      </Flex>
    </Flex>
  );
};
