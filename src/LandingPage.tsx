import {Box, Flex, Image, Space, Stack, Text} from "@mantine/core";
import {open} from '@tauri-apps/plugin-dialog';
import React, {useEffect} from "react";
import {Effect, getCurrentWindow, LogicalSize} from "@tauri-apps/api/window"
import {set_directory} from "./commands.tsx";
const appWindow = getCurrentWindow()

const LandingPage = ({setDirectory}: { setDirectory: React.Dispatch<React.SetStateAction<string>> }) => {
    useEffect(() => {
        appWindow.setSize(new LogicalSize(600, 450)).then(null);
        appWindow.setEffects({effects: [Effect.Blur], radius: 10}).then(null);
    }, [])

    const chooseFolder = async () => {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: "C:/"
        });
        console.log(selected);
        if (!(Array.isArray(selected) || selected === null)) {
            let dir = selected.replace(/\\/g, "/")
            if (await set_directory(dir)) {
                setDirectory(dir);
            }
        }
    }

    return (
        <Flex direction={"column"} align={"center"} style={{userSelect: "none"}}>
            <Text c={"#dee0e2"} fw={500} mt={50} fz={40}>
                Welcome to CP-Assist
            </Text>
            <Space h={"md"}/>
            <Text c={"#6f737a"} fz={"md"}>
                Create a new project to start from scratch or open existing folder.
            </Text>
            <Space h={"lg"}/>
            <Flex>
                <Stack m={40}>
                    <Box bg={"#2b2d30"} p={15} style={{borderRadius: 7, cursor: 'pointer'}} onClick={chooseFolder}>
                        <Image src={"/add.svg"} w={35} m={"auto"}/>
                    </Box>
                    <Text fz={"xs"} c={"#d5dee1"}>New Project</Text>
                </Stack>
                <Stack m={40}>
                    <Box bg={"#2b2d30"} p={15} style={{borderRadius: 7, cursor: 'pointer'}} onClick={chooseFolder}>
                        <Image src={"/folder.svg"} w={35} m={"auto"}/>
                    </Box>
                    <Text fz={"xs"} c={"#d5dee1"}>Open Existing</Text>
                </Stack>
            </Flex>
        </Flex>
    );
}

export default LandingPage;