import { useDevices } from "../hooks";
import { connect_device } from "../commands";
import { Combobox, InputBase, Input, useCombobox, Group, Text, Indicator } from '@mantine/core';
import { PeripheralResult } from "../types";
import { useEffect } from "react";

interface DeviceListProps {
    value?: string | null
    onChange(id: string | null): void
}


function DeviceOption({ id, name, connected }: PeripheralResult) {
    return (
        <Group>
            <Text fz="sm" fw={500}>
                {name ?? id}
            </Text>
            {connected && <Indicator color="green" />}
        </Group>
    );
}

export function DeviceList({ onChange, value }: DeviceListProps) {
    const combobox = useCombobox({
        onDropdownClose: () => combobox.resetSelectedOption(),
    });

    const devices = useDevices();
    const selectedOption = devices.data?.find((item) => item.id === value);
    const options = devices.data?.map((item) => (
        <Combobox.Option value={item.id} key={item.id}>
            <DeviceOption {...item} />
        </Combobox.Option>
    ));
    useEffect(() => {
        if (value) {
            const device = devices.data?.find((item) => item.id === value);
            if (device && !device.connected) {
                connect_device(device.id)
            }
        }
    }, [devices.data, selectedOption?.id, value])
    return (
        <Combobox

            store={combobox}
            withinPortal={false}
            onOptionSubmit={(val) => {
                onChange(val);
                combobox.closeDropdown();
            }}
        >
            <Combobox.Target>
                <InputBase
                    component="button"
                    type="button"
                    pointer
                    styles={
                        {
                            root: {
                                flexGrow: 1,
                                flexWrap: "nowrap"
                            }
                        }}
                    rightSection={<Combobox.Chevron />}
                    onClick={() => combobox.toggleDropdown()}
                    rightSectionPointerEvents="none"
                    multiline
                >
                    {selectedOption ? (
                        <DeviceOption {...selectedOption} />
                    ) : (
                        <Input.Placeholder>Connect to Device</Input.Placeholder>
                    )}
                </InputBase>
            </Combobox.Target>

            <Combobox.Dropdown>
                <Combobox.Options>{options}</Combobox.Options>
            </Combobox.Dropdown>
        </Combobox>
    );
}