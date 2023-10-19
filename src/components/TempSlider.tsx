import { Slider, rem } from "@mantine/core";
import { setTemperature } from "../hooks/useSetTemp";
import { OperatingMode, ParsedDeviceStatus } from "../types";
import { CtoF } from "../util";
import { useMantineTheme } from '@mantine/core';
import { useElementSize } from '@mantine/hooks';
import { useState } from "react";
import chroma from "chroma-js"
//const SliderMarks = Object.values(TempRanges).map(val => ({ value: val.range.min, label: val.mode }))

interface TempSliderProps {
    bedjet: string,
    data?: ParsedDeviceStatus
}
export default function TempSlider({ bedjet, data }: TempSliderProps) {
    const theme = useMantineTheme();
    const { ref, width } = useElementSize();
    const [selectedValue, setSelectedValue] = useState(Math.round(CtoF(data?.target_temp ?? 0)))
    if (!data) {
        return (
            <Slider disabled={true} />
        )
    }
    const min = 66
    const max = 92
    // const actualPercent = (actualValue - min) / (max - min)
    const selectedPercent = (selectedValue - min) / (max - min);

    const blue = theme.colors.blue[6]
    const red = theme.colors.red[6]

    const gradientStep = chroma.scale([blue, red]).mode("hcl").padding(-0.75);
    return (
        <Slider
            disabled={data.operating_mode === OperatingMode.TurboHeat || data.operating_mode === OperatingMode.NormalHeat}
            min={66}
            max={92}
            marks={
                [
                    {
                        value: Number(CtoF(data.actual_temp).toFixed(1)),
                        label: `Actual: ${CtoF(data.actual_temp).toFixed(1)}°F`
                    },
                ]
            }
            ref={ref}
            labelAlwaysOn
            defaultValue={Math.round(CtoF(data.target_temp))}
            label={(label) => `${label} °F`}
            onChange={(val) => setSelectedValue(val)}
            onChangeEnd={(value) => {
                setTemperature(bedjet, data, value)
            }}
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
                    backgroundColor: `${gradientStep(selectedPercent)}`,
                    height: rem(28),
                    width: rem(38),
                    border: "none"
                },
               // markWrapper: { transition: `left 0.3s ease-out` },
                bar: {
                    background: `linear-gradient(to right, ${gradientStep.colors(10)})`,
                    backgroundSize: `${width}px 100%`,
                    backgroundPosition: `-${selectedPercent}px 0`,
                    backgroundRepeat: "no-repeat"
                }
            }}
        />
    )
}