import { SegmentedControl } from "@mantine/core";
import { ButtonCode, OperatingMode, ParsedDeviceStatus } from "../types";
import { setTemperature } from "../hooks/useSetTemp";
import { send_command } from "../commands";
import { useSyncedState } from "../hooks";

interface ModeControlProps {
    bedjet: string,
    data?: ParsedDeviceStatus
}

const ModeValues = ["Off", "Normal", "Heat", "Turbo"] as const;
type Mode = typeof ModeValues[number]

function getMode(data?: ParsedDeviceStatus): Mode | undefined {
    if (!data) return undefined

    if (data.operating_mode === OperatingMode.Standby) return "Off"
    if (data.operating_mode === OperatingMode.Cool) return "Normal"
    if (data.operating_mode === OperatingMode.Dry) return "Normal"
    if (data.operating_mode === OperatingMode.ExtendedHeat) return "Normal"
    if (data.operating_mode === OperatingMode.NormalHeat) return "Heat"
    if (data.operating_mode === OperatingMode.TurboHeat) return "Turbo"

    return "Off"
}

export function ModeControl({ bedjet, data }: ModeControlProps) {
    const [value, setValue] = useSyncedState(undefined, getMode(data), bedjet)

    return (
        <SegmentedControl
            value={value}
            data={[...ModeValues]}
            onChange={(val) => {
                const mode = val as Mode
                if (mode === "Off") {
                    send_command(bedjet, {
                        type: "SetTime",
                        content: {
                            hours: 0,
                            minutes: 0,
                        },
                    })
                }
                if (mode === "Normal" && data) {
                    setTemperature(bedjet, data, { type: "Celsius", value: data.target_temp })
                }
                if (mode === "Heat") {
                    send_command(bedjet, { type: "Button", content: ButtonCode.Heat })
                }

                if (mode === "Turbo") {
                    send_command(bedjet, { type: "Button", content: ButtonCode.Turbo })
                }

                setValue(val as Mode)
            }}
        />
    )
}