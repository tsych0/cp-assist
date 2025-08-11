import "./App.css";
import { Box } from "@mantine/core";
import TitleBar from "./Titlebar.tsx";
import LandingPage from "./LandingPage.tsx";
import { useEffect, useState } from "react";
import Home from "./Home.tsx";
import { Problem, Verdict } from "./Languages.ts";
import { listen } from "@tauri-apps/api/event";
import {
  get_directory,
  get_problem,
  get_verdicts,
  set_problem,
  set_verdicts,
  run,
  submit
} from "./commands.tsx";

function App() {
  const [directory, setDirectory] = useState("");
  const [problem, setProblem] = useState<Problem | null>(null);
  const [verdicts, setVerdicts] = useState<Verdict[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    get_directory().then((dir) => setDirectory(dir));
    get_problem().then((pro) => setProblem(pro));
    get_verdicts().then((ver) => setVerdicts(ver || []));
    listen<number>("test", async (event) => {
      if (!loading) {
        setLoading(true);
        await run();
        setLoading(false);
      }
    })
    listen<number>("submit", async (event) => {
      await submit();
    })
    listen<Problem>("set-problem", (event) =>
      set_problem(event.payload).then(() => setProblem(event.payload)),
    );
    listen<Verdict[]>("set-verdicts", (event) =>
      set_verdicts(event.payload).then(() => setVerdicts(event.payload)),
    );
  }, []);

  return (
    <Box
      className="bg-[#1e1f22]/[80%] border border-[#3c3f41]"
      style={{ height: "100%", width: "100%", position: "fixed",
      borderRadius: "15px"}}
    >
      <TitleBar setDirectory={setDirectory} directory={directory} loading={loading} setLoading={setLoading} />
      {directory === "" && <LandingPage setDirectory={setDirectory} />}
      {directory !== "" && <Home problem={problem} verdicts={verdicts} />}
    </Box>
  );
}

export default App;
