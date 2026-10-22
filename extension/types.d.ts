
interface Settings {
  telegramToken: string;
  chatId: string;
  htmlSelector: string;
  hotkeyCode: string;
  hotkeyCtrl: boolean;
  hotkeyAlt: boolean;
  hotkeyShift: boolean;
  sendPhoto: boolean;
}

interface KeydownForward {
  type: "hrb-keydown";
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  repeat: boolean;
  typing: boolean;
}

interface ExtractRequest {
  type: "hrb-extract";
  selector: string;
}
