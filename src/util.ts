export function secondsToHHMM(seconds: number) {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);

  const hDisplay = h < 10 ? "0" + h : h;
  const mDisplay = m < 10 ? "0" + m : m;

  return hDisplay + ":" + mDisplay;
}

export const CtoF = (temp: number) => temp * 9 / 5 + 32;
export const FtoC = (temp: number) => (temp - 32) * 5 / 9;
