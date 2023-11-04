import { Group, Text } from "@mantine/core";

type IconLabelProps = {
    icon: React.ReactNode;
    label: string;
}

export function IconLabel({ icon, label }: IconLabelProps) {
    return (
        <Group gap="xs">
            {icon}
            <Text size="sm">{label}</Text>
        </Group>
    )
}