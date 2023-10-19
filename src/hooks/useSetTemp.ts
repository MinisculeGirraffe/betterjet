import { send_command } from "../queries";
import { ButtonCode, OperatingMode, ParsedDeviceStatus } from "../types";

type TempRange = { min: number; max: number };

export const TempRanges: { mode: OperatingMode; range: TempRange }[] = [
  { mode: OperatingMode.Cool, range: { min: 66, max: 79 } },
  { mode: OperatingMode.Dry, range: { min: 80, max: 89 } },
  { mode: OperatingMode.ExtendedHeat, range: { min: 90, max: 92 } },
];

export const ModeButtonMapping: { [key in OperatingMode]?: ButtonCode } = {
  [OperatingMode.Cool]: ButtonCode.Cool,
  [OperatingMode.Dry]: ButtonCode.Dry,
  [OperatingMode.ExtendedHeat]: ButtonCode.ExternalHeat,
};

export async function setTemperature(
  id: string,
  deviceStatus: ParsedDeviceStatus,
  targetTemp: number,
): Promise<void> {
  let requiredMode: OperatingMode | undefined;

  for (const { mode, range } of TempRanges) {
    if (targetTemp >= range.min && targetTemp <= range.max) {
      requiredMode = mode;
      break;
    }
  }

  if (requiredMode === undefined) {
    throw new Error("Invalid target temperature");
  }

  const requiredButton: ButtonCode | undefined =
    ModeButtonMapping[requiredMode];

  if (requiredButton === undefined) {
    throw new Error("Invalid operating mode");
  }

  if (deviceStatus.operating_mode !== requiredMode) {
    // save the current timer and fan step

    const originalFanStep = deviceStatus.fan_step;

    // switch the mode
    await send_command(id, { type: "Button", content: requiredButton });
    if (deviceStatus.remaining_duration != 0) {
      await send_command(id, {
        type: "SetTime",
        content: {
          hours: Math.floor(deviceStatus.remaining_duration / 3600),
          minutes: Math.floor((deviceStatus.remaining_duration % 3600) / 60),
        },
      });
    }

    await send_command(id, {
      type: "SetFan",
      content: { type: "Percent", value: originalFanStep },
    });
  }

  // set the temperature

  await send_command(id, {
    type: "SetTemp",
    content: { "type": "Fahrenheit", value: targetTemp },
  });
}

