import { Group, Slider, rem, Text } from "@mantine/core"
import { ParsedDeviceStatus } from "../types"
import { send_command } from "../queries"
import { IconPropeller } from '@tabler/icons-react';

import { useState } from "react";
interface FanSliderProps {
    bedjet: string,
    data?: ParsedDeviceStatus
}

export default function FanSlider({ data, bedjet }: FanSliderProps) {
    const [value, setValue] = useState(data?.fan_step ?? 5)
    if (!data) return (
        <Slider disabled={true} />
    )

    return (
        <>
            <Group>
                <IconPropeller
                />
                <Text>Fan</Text>
            </Group>
            <Slider
                min={5}
                max={100}
                step={5}
                value={value}
                labelAlwaysOn
                defaultValue={data?.fan_step}
                label={label => `${label}%`}
                onChange={(value) => setValue(value)}
                onChangeEnd={(value) =>
                    send_command(bedjet, { type: "SetFan", content: { type: "Percent", value } })
                }
                marks={[{ value: 5, label: "5%" }, { value: 100, label: "100%" }]}
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
        </>
    )
}