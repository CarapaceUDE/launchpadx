import { LauncherProvider } from "./context/LauncherContext";
import { LauncherPage } from "./pages/LauncherPage";

function App() {
  return (
    <LauncherProvider>
      <LauncherPage />
    </LauncherProvider>
  );
}

export default App;