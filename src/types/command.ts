export enum CapturedType {
  Request = 'request',
  Response = 'response',
}

export interface Captured {
  id: string;
  type: CapturedType;
  content: string;
}
