// Duplicates src/config.rs `Settings` (camelCase via serde rename_all).
// Keep in sync when renaming fields there.
export interface Settings {
  telegramToken: string;
  chatId: string;
  htmlSelector: string;
  hotkeyCode: string;
  hotkeyCtrl: boolean;
  hotkeyAlt: boolean;
  hotkeyShift: boolean;
  sendPhoto: boolean;
}

export interface KeydownForward {
  type: "hrb-keydown";
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  repeat: boolean;
  typing: boolean;
}

export interface ExtractRequest {
  type: "hrb-extract";
  selector: string;
}
