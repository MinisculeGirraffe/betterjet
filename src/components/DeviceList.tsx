import { Loader, Select } from "@mantine/core";
import { useDevices } from "../hooks";
import { connect_device, disconnect_device } from "../queries";
import { useQueryClient } from "@tanstack/react-query";

interface DeviceListProps {
    value?: string | null
    onChange(id: string | null): void
}
export function DeviceList({ onChange, value }: DeviceListProps) {
    const devices = useDevices();
    const client = useQueryClient();
    const list = devices.data ?? []

    if (devices.isLoading) return <Loader />
    console.log(list)
    return (
        <Select
            checkIconPosition="right"
            value={value}
            data={list.map(i => ({ value: i.id, label: i.name ?? i.id }))}
            onClick={() => devices.refetch()}
            onChange={(valnew) => {
                valnew === null ? disconnect_device(value ?? "") : connect_device(valnew);
                client.invalidateQueries(["devices"]);
                onChange(valnew)
            }}
        />


    )
}