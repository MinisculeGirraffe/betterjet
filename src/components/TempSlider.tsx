import { Slider, rem } from "@mantine/core";
import { setTemperature } from "../hooks/useSetTemp";
import { OperatingMode, ParsedDeviceStatus } from "../types";
import { CtoF } from "../util";
import { useMantineTheme } from '@mantine/core';
import { useState } from "react";
import chroma from "chroma-js"
import css from "./TempSlider.module.css"

interface TempSliderProps {
    bedjet: string,
    data?: ParsedDeviceStatus
}
export default function TempSlider({ bedjet, data }: TempSliderProps) {
    const theme = useMantineTheme();
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
            classNames={{
                markWrapper: css.markWrapper,
                mark: css.mark,
                markLabel: css.markLabel,
                track: css.track,
                bar: css.bar,
                thumb: css.thumb,
                label: css.label,
                root: css.root,
                trackContainer: css.trackContainer
            }}
            marks={
                [
                    {
                        value: Number(CtoF(data.actual_temp).toFixed(1)),
                        label: `Actual: ${CtoF(data.actual_temp).toFixed(1)}°F`
                    },
                ]
            }
            labelAlwaysOn
            defaultValue={Math.round(CtoF(data.target_temp))}
            label={(label) => `${label} °F`}
            onChange={(val) => setSelectedValue(val)}
            onChangeEnd={(value) => {
                setTemperature(bedjet, data, value)
            }}
            unstyled
            styles={{
                thumb: {
                    backgroundColor: `${gradientStep(selectedPercent)}`,
                    height: rem(28),
                    width: rem(38),
                    border: "none"
                },
                bar: {
                    background: "none"
                },
            }}
        />
    )
}