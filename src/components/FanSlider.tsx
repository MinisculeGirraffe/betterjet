import { Slider, rem } from "@mantine/core"
import { ParsedDeviceStatus } from "../types"
import { send_command } from "../queries"

interface FanSliderProps {
    bedjet: string,
    data?: ParsedDeviceStatus
}

export default function FanSlider({ data, bedjet }: FanSliderProps) {
    if(!data) return (
        <Slider disabled={true} />
    )
    return (
        <Slider
            min={5}
            max={100}
            step={5}
            labelAlwaysOn
            defaultValue={data?.fan_step}
            label={label => `${label}%`}
            onChangeEnd={(value) =>
                send_command(bedjet, { type: "SetFan", content: { type: "Percent", value } })
            }
            styles={{
                label: {
                    top: 0,
                    height: rem(28),
                    width: rem(38),
                    lineHeight: rem(28),
                    padding: 0,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontWeight: 700,
                    backgroundColor: "transparent"

                },
                thumb: {
                    background: "var(--mantine-color-blue-6)",
                    height: rem(28),
                    width: rem(38),
                    border: "none"
                },
            }}
        />
    )
}