import { MantineTheme, Slider, rem, useComputedColorScheme } from "@mantine/core";
import { setTemperature } from "../hooks/useSetTemp";
import { OperatingMode, ParsedDeviceStatus, TemperatureUnit, UserPreferences } from "../types";
import { CtoF, FtoC } from "../util";
import { useMantineTheme } from '@mantine/core';
import chroma from "chroma-js"
import css from "./TempSlider.module.css"
import { useConfig, useSyncedState } from "../hooks";

const percent = (value: number, min: number, max: number) => (value - min) / (max - min);
const convert = (temp: number, config: UserPreferences) => config?.unit === TemperatureUnit.Celsius ? temp : CtoF(temp);
const toFixed = (temp: number) => Number(temp.toFixed(1));
const convertFixed = (temp: number, config: UserPreferences) => toFixed(convert(temp, config));

const getPrimaryShade = (theme: MantineTheme, colorScheme: "light" | "dark") =>
    typeof theme.primaryShade === "number"
        ? theme.primaryShade
        : theme.primaryShade[colorScheme];

const getTempSymbol = (unit?: TemperatureUnit) =>
    unit ?? TemperatureUnit.Celsius === TemperatureUnit.Celsius
        ? "°C"
        : "°F";

interface TempSliderProps {
    bedjet: string,
    data?: ParsedDeviceStatus
}

export default function TempSlider({ bedjet, data }: TempSliderProps) {
    const config = useConfig().data;
    const [value, setValue] = useSyncedState(undefined, convertFixed(data?.target_temp ?? 0, config!), bedjet)

    const theme = useMantineTheme()
    const computedColorScheme = useComputedColorScheme('light');
    const shade = getPrimaryShade(theme, computedColorScheme);
    const blue = theme.colors.blue[shade]
    const red = theme.colors.red[shade]
    const gradientStep = chroma.scale([blue, red]).mode("hcl").padding(-0.75);

    const tempSymbol = getTempSymbol(config?.unit)

    if (!value || !data || !config) {
        return (
            <Slider disabled={true} />
        )
    }

    const min = convertFixed(19, config)
    const max = convertFixed(33.5, config)
    const selectedPercent = percent(value, min, max);
    return (
        <Slider
            disabled={data.operating_mode === OperatingMode.TurboHeat || data.operating_mode === OperatingMode.NormalHeat}
            min={min}
            max={max}
            step={config.unit === TemperatureUnit.Celsius ? 0.5 : 1}
            vars={() => ({ "root": { "--slider-color": gradientStep(selectedPercent).hex() } })}
            classNames={{
                track: css.track,
                mark: css.mark,
            }}
            marks={
                [
                    {
                        value: convertFixed(data.actual_temp, config),
                        label: `Actual: ${convertFixed(data.actual_temp, config)}${tempSymbol}`
                    },
                ]
            }
            labelAlwaysOn
            value={value}
            label={(label) => `${config.unit === TemperatureUnit.Celsius ? label : Math.round(label)}${tempSymbol}`}
            onChange={(val) => setValue(val)}
            onChangeEnd={(value) => {
                setTemperature(bedjet, data, { type: "Celsius", value: config.unit === TemperatureUnit.Celsius ? value : FtoC(value) })
            }}
            styles={{
                root: {
                    "--gradient": `linear-gradient(to right, ${gradientStep.colors(10)})`
                } as React.CSSProperties,
                thumb: {
                    backgroundColor: `${gradientStep(selectedPercent)}`,
                    height: rem(28),
                    width: rem(38),
                    border: "none"
                },
                bar: {
                    background: "none"
                },
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
            }}
        />
    )
}