export function secondsToHHMM(seconds: number) {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds - (hours * 3600)) / 60);
  const secs = seconds - (hours * 3600) - (minutes * 60);

  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
}

export const CtoF = (temp: number) => temp * 9 / 5 + 32;
export const FtoC = (temp: number) => (temp - 32) * 5 / 9;
