// components/Settings/Settings.tsx
import { Flex, NumberInput, Paper, Switch, Text } from '@mantine/core';
import React from 'react';
import { KeySettingInput } from '../../../../components/KeySettingInput';
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
  autoFollowLeaderKey,
  setAutoFollowLeaderKey,
  watchingAutoFollowLeaderKey,
  watchAutoFollowLeaderKey,
  focusChatKey,
  setFocusChatKey,
  watchingFocusChatKey,
  watchFocusChatKey,
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
        maxHeight: '90vh',
        overflowY: 'auto',
        overflowX: 'hidden',
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
          <KeySettingInput
            label="Auto Follow Dofus key"
            value={autoFollowLeaderKey}
            onSet={watchAutoFollowLeaderKey}
            onClear={() => setAutoFollowLeaderKey('')}
            watching={watchingAutoFollowLeaderKey}
          />
          <KeySettingInput
            label="Focus Chat Dofus key"
            value={focusChatKey}
            onSet={watchFocusChatKey}
            onClear={() => setFocusChatKey('')}
            watching={watchingFocusChatKey}
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
        <ShortcutSettingInput
          label="Follow leader"
          shortcutType="auto_follow_leader"
          shortcutKey={shortcuts.auto_follow_leader}
          onRegister={registerShortcut}
          onRemove={removeShortcut}
          watching={watchingInput === 'auto_follow_leader'}
        />
        <ShortcutSettingInput
          label="Auto invite all"
          shortcutType="auto_invite_all"
          shortcutKey={shortcuts.auto_invite_all}
          onRegister={registerShortcut}
          onRemove={removeShortcut}
          watching={watchingInput === 'auto_invite_all'}
        />
      </Flex>
    </Flex>
  );
};
