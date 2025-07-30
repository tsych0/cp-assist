import {
  Box,
  Center,
  Flex,
  Group,
  Select,
  Stack,
  Text,
  Textarea,
} from "@mantine/core";
import { useEffect, useState } from "react";
import { LogicalSize } from "@tauri-apps/api/window";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Problem, Verdict } from "./Languages.ts";
import { IconPlus } from "@tabler/icons-react";
import { emit } from "@tauri-apps/api/event";

const appWindow = getCurrentWebviewWindow();

const MyTextarea = ({
  title,
  value,
  mx,
  onChange,
  onClick,
}: {
  title: string;
  value: string;
  mx: number | string | undefined;
  onChange: (v: string) => void;
  onClick?: () => void;
}) => {
  return (
    <Box
      h={"90%"}
      w={"31%"}
      fw={500}
      bg={"#282828"}
      className={`text-3xl border border-gray-600 relative text-center select-none font-mono
                            rounded-md tracking-widest pt-1`}
      mx={mx}
    >
      <div onClick={onClick} style={{ cursor: onClick ? "pointer" : "unset" }}>
        {title}
      </div>
      <Textarea
        className={`h-[91%] w-full px-2 top-10 bg-[#3e3e3e]/50 rounded-md absolute`}
        variant="unstyled"
        maxRows={15}
        autosize
        onChange={(v) => onChange(v.currentTarget.value)}
        value={value}
      />
    </Box>
  );
};

const Home = ({
  problem,
  verdicts,
}: {
  problem: Problem | null;
  verdicts: Verdict[];
}) => {
  let [caseIndex, setCaseIndex] = useState(0);
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [stdError, setError] = useState("");
  const [answer, setAnswer] = useState("");
  const [editable, setEditable] = useState(false);
  const [showError, setShowError] = useState(false);
  const otherVerdict =
    verdicts.filter((v) => v.status_id !== 3)[0] || verdicts[caseIndex];
  const cases = Array.from(Array(verdicts.length).keys())
    .map((x) => ({
      label:
        `Case ${x + 1} ` +
        (verdicts[x]?.status_id === 3
          ? "✔️"
          : verdicts[x]?.status_id < 3
            ? ""
            : "❌"),
      value: `${x + 1}`,
    }))
    .concat({ label: "Add Case", value: "-1" });

  const onCaseChange = (v: string | null): void => {
    let index = parseInt(v ?? "1") - 1;
    if (index == -2) {
      setEditable(true);
    } else {
      setEditable(false);
    }
    setCaseIndex(index);
    setInput(verdicts[index]?.input ?? "");
    setOutput(verdicts[index]?.output ?? "");
    setError(verdicts[index]?.stderr ?? "");
    setAnswer(verdicts[index]?.answer ?? "");
  };

  const onEdit = (setAny: (v: string) => void) => {
    if (!editable) return (_: string) => {};
    return setAny;
  };

  const onAddTestCase = () => {
    if (!editable) return;
    verdicts.push({
      input: input,
      answer: answer,
      output: "",
      stderr: "",
      memory: 0,
      status: "NA",
      status_id: 0,
      time: 0,
    });
    emit("set-verdicts", verdicts);
  };

  useEffect(() => {
    setInput(verdicts[caseIndex]?.input ?? "");
    setOutput(verdicts[caseIndex]?.output ?? "");
    setError(verdicts[caseIndex]?.stderr ?? "");
    setAnswer(verdicts[caseIndex]?.answer ?? "");
    appWindow.setSize(new LogicalSize(1000, 650)).then(null);
  }, [verdicts]);

  return problem?.title ? (
    <Stack h={"95%"}>
      <Flex className={"mt-5 justify-center content-center"}>
        <Text
          w={500}
          fz={32}
          fw={600}
          c={"white"}
          className={"select-none tracking-widest text-center"}
        >
          {problem?.title}
        </Text>
      </Flex>

      <Stack
        c={"white"}
        w={"95%"}
        mx={"auto"}
        px={40}
        className={"text-2xl h-full rounded-md"}
      >
        {/*Heading Area*/}
        <Group my={0}>
          <Select
            variant="unstyled"
            onChange={onCaseChange}
            data={cases}
            w={150}
            pl={10}
            py={4}
            bg={"#2b2d30"}
            className={"rounded-md"}
            checkIconPosition={"right"}
            defaultValue={"1"}
            allowDeselect={false}
          />
          {editable && (
            <IconPlus
              onClick={onAddTestCase}
              className="ml-3 cursor-pointer border-white/15 border rounded-full"
              title="Save Test Case"
            />
          )}
          {verdicts[caseIndex]?.time && (
            <Group mx={10}>
              <Text ff={"monospace"} c={"#fcfcfc"} fz={20}>
                {verdicts[caseIndex]?.time + " ms"}
              </Text>
              <Text ff={"monospace"} c={"#fcfcfc"} fz={20}>
                {Math.floor(verdicts[caseIndex]?.memory / 1024) + " Mb"}
              </Text>
            </Group>
          )}

          {!editable && (
            <Text
              fz={26}
              fw={600}
              ml={"md"}
              my={"auto"}
              ff={"monospace"}
              className={"tracking-wider"}
              style={{
                color:
                  otherVerdict?.status_id === 3
                    ? "#2cad40"
                    : otherVerdict?.status_id < 3
                      ? "gray"
                      : "red",
              }}
            >
              {otherVerdict?.status}
            </Text>
          )}
        </Group>

        <Group h={"90%"} w={"100%"}>
          <MyTextarea
            title={"Input"}
            value={input}
            mx={0}
            onChange={(v) => onEdit(setInput)(v)}
          />
          <MyTextarea
            title={"Answer"}
            value={answer}
            mx={"auto"}
            onChange={(v) => onEdit(setAnswer)(v)}
          />
          <MyTextarea
            title={showError ? "Error" : "Output"}
            value={showError ? stdError : output}
            onChange={() => {}}
            onClick={() => setShowError(!showError)}
            mx={0}
          />
        </Group>
      </Stack>
    </Stack>
  ) : (
    <Center h={"90%"}>
      <Text c={"#acacac"} fz={32} className={"tracking-wider"}>
        Select problem from competitive companion
      </Text>
    </Center>
  );
};

export default Home;
