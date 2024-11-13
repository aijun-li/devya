import { toast } from 'vue-sonner';

export function handleError(error: unknown) {
  let message = '';

  if (typeof error === 'string') {
    message = error;
  } else if (error instanceof Error) {
    message = error.message;
  } else {
    message = JSON.stringify(error);
  }

  toast.error(message);
}
