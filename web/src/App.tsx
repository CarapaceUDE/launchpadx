import { LaunchPadXProvider } from "./context/LaunchPadXContext";
import { LauncherPage } from "./pages/LauncherPage";

function App() {
  return (
    <LaunchPadXProvider>
      <LauncherPage />
    </LaunchPadXProvider>
  );
}

export default App;