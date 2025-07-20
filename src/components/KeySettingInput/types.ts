// components/KeySettingInput/types.ts
export type KeySettingInputProps = {
  label: string;
  value: string;
  onSet: () => void;
  onClear: () => void;
  watching: boolean;
};
