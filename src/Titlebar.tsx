import { Center, Flex, Group, Image, Select, Text } from "@mantine/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  create_file,
  get_language,
  get_languages,
  read_config,
  run,
  set_directory,
  set_language,
  submit,
} from "./commands.tsx";
import { IconCheck } from "@tabler/icons-react";

const appWindow = getCurrentWindow();

const TitleBar = ({
  directory,
  setDirectory,
  loading,
  setLoading,
}: {
  directory: string;
  setDirectory: (arg0: string) => void;
  loading: boolean;
  setLoading: (arg: boolean) => void;
}) => {
  const [isFocused, setIsFocused] = useState(true);
  const [language, setLanguage] = useState("0");
  const [languages, setLanguages] = useState<
    { value: string; label: string }[]
  >([]);
  const trimmedLanguages = useMemo(() => {
    return languages.map((v) => {
      return { label: v.label.split("(")[0], value: v.value };
    });
  }, [languages]);

  const languageFromId = (id: string) =>
    languages.filter((v) => v.value === id)[0].label;

  const onChangeLanguage = async (value: string | null) => {
    if (value === null) return;
    let success = await set_language(parseInt(value));
    if (success) setLanguage(value);
  };

  const onRun = async () => {
    setLoading(true);
    await run();
    setLoading(false);
  };

  useEffect(() => {
    getCurrentWindow().onCloseRequested(() => {
      invoke("save_state")
        .then(null)
        .catch((e) => console.error(e));
    });
    get_language().then((v) => setLanguage(v.toString()));
    get_languages().then((v) =>
      setLanguages(
        v.map((x) => {
          return { value: x.id.toString(), label: x.name };
        }),
      ),
    );

    read_config();
    window.addEventListener("focus", () => setIsFocused(true));
    window.addEventListener("blur", () => setIsFocused(false));
  }, []);

  return (
    <Flex
      h={40}
      data-tauri-drag-region
      style={{ backgroundColor: isFocused ? "#2b2d30" : "#3c3f41" }}
    >
      {directory !== "" && (
        <>
          <Center
            h={33}
            w={33}
            mx={3}
            my={"auto"}
            className="rounded-full hover:bg-[#484b4d] z-10"
            onClick={() => {
              set_directory("").then(() => setDirectory(""));
            }}
          >
            <Image src="back.svg" h={25} />
          </Center>

          <Flex w={"100%"} h={40} pos={"absolute"} data-tauri-drag-region>
            <Flex my={"auto"} mx={"auto"}>
              {loading && (
                <Center
                  h={35}
                  w={180}
                  my={"auto"}
                  mx={5}
                  className="rounded-md bg-[#484b4d] cursor-pointer"
                >
                  <Image src="pending.gif" h={60} />
                  <Text mx={10} c={"white"}>
                    Running
                  </Text>
                </Center>
              )}
              {!loading && (
                <>
                  <Center
                    h={35}
                    w={80}
                    my={"auto"}
                    mx={5}
                    className="bg-black/15 rounded-md hover:bg-[#484b4d] cursor-pointer"
                    onClick={onRun}
                  >
                    <Image src="play.svg" h={25} ml={10} />
                    <Text mx={10} c={"white"}>
                      Run
                    </Text>
                  </Center>
                  <Center
                    h={35}
                    my={"auto"}
                    w={115}
                    className="bg-black/15 rounded-md hover:bg-[#484b4d] cursor-pointer"
                    onClick={() => submit()}
                  >
                    <Image src="submit.svg" h={25} ml={10} />
                    <Text mx={10} c={"#28c244"}>
                      Submit
                    </Text>
                  </Center>
                </>
              )}
            </Flex>
          </Flex>
        </>
      )}

      <Flex ml={"auto"} />

      {directory !== "" && (
        <>
          <Center
            h={34}
            w={34}
            my={"auto"}
            mx={2}
            title={"Create File"}
            className="rounded-md hover:bg-[#484b4d] cursor-pointer z-10"
            onClick={() => create_file()}
          >
            <Image src="create_file.svg" h={24} fit="contain" />
          </Center>

          <Select
            h={35}
            my={"auto"}
            w={160}
            variant="light"
            c={"white"}
            defaultValue={"0"}
            data={trimmedLanguages}
            value={language}
            className={"bg-black/15 rounded-md z-10"}
            onChange={onChangeLanguage}
            allowDeselect={false}
            comboboxProps={{ width: 250 }}
            renderOption={({ option, checked }) => (
              <Group>
                <Text fz={18} fw={"500"} className={"tracking-wider"}>
                  {languageFromId(option.value)}
                </Text>{" "}
                {checked && (
                  <IconCheck style={{ marginInlineStart: "auto" }} />
                )}{" "}
              </Group>
            )}
          />
        </>
      )}

      <Flex ml={10} style={{ zIndex: 3 }}>
        <Center
          h={40}
          w={36}
          className="hover:bg-[#484b4d]"
          onClick={() => appWindow.minimize()}
        >
          <Image src="minimize.svg" fit="contain" h={22} />
        </Center>
        <Center h={40} w={36}>
          <Image src="maximize.svg" fit="contain" h={22} />
        </Center>
        <Center
          h={40}
          w={36}
          className="hover:bg-[#e81123]"
          onClick={() => appWindow.close()}
        >
          <Image src="close.svg" h={22} fit="contain" />
        </Center>
      </Flex>
    </Flex>
  );
};

export default TitleBar;
