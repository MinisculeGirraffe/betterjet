import { send_command } from "../commands";
import {
  ButtonCode,
  OperatingMode,
  ParsedDeviceStatus,
  TempParam,
} from "../types";
import { FtoC } from "../util";

type TempRange = { min: number; max: number };

export const TempRanges: { mode: OperatingMode; range: TempRange }[] = [
  { mode: OperatingMode.Cool, range: { min: 19, max: 25 } },
  { mode: OperatingMode.Dry, range: { min: 25, max: 30 } },
  { mode: OperatingMode.ExtendedHeat, range: { min: 30, max: 33.5 } },
];

export const ModeButtonMapping: { [key in OperatingMode]?: ButtonCode } = {
  [OperatingMode.Cool]: ButtonCode.Cool,
  [OperatingMode.Dry]: ButtonCode.Dry,
  [OperatingMode.ExtendedHeat]: ButtonCode.ExternalHeat,
};

export async function setTemperature(
  id: string,
  deviceStatus: ParsedDeviceStatus,
  param: TempParam,
): Promise<void> {
  let requiredMode: OperatingMode | undefined;
  const targetTemp = Math.max(
    Math.min(
      param.type === "Fahrenheit" ? FtoC(param.value) : param.value,
      33.5,
    ),
    19,
  );
  console.log(targetTemp);
  for (const { mode, range } of TempRanges) {
    if (targetTemp >= range.min && targetTemp <= range.max) {
      requiredMode = mode;
      break;
    }
  }
  console.log(requiredMode);

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
    content: { "type": "Celsius", value: targetTemp },
  });
}
