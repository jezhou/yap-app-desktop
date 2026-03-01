import { useState, useEffect } from "react";
import {
  BrowserRouter,
  Routes,
  Route,
  useNavigate,
} from "react-router-dom";
import Sidebar from "./components/layout/Sidebar";
import MainContent from "./components/layout/MainContent";
import SessionsPage from "./pages/SessionsPage";
import SessionDetailPage from "./pages/SessionDetailPage";
import ConversationDetailPage from "./pages/ConversationDetailPage";
import SettingsPage from "./pages/SettingsPage";
import SetupWizard from "./components/settings/SetupWizard";
import { listAvailableModels } from "./services/settings";

function AppLayout() {
  return (
    <div className="flex h-screen bg-bg">
      <Sidebar />
      <MainContent />
    </div>
  );
}

function SetupGuard({ children }: { children: React.ReactNode }) {
  const navigate = useNavigate();
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    async function checkModels() {
      try {
        const models = await listAvailableModels();
        const hasDownloaded = models.some((m) => m.downloaded);
        if (!hasDownloaded) {
          navigate("/setup", { replace: true });
        }
      } catch {
        // If we can't check models, proceed normally
      } finally {
        setChecking(false);
      }
    }
    checkModels();
  }, [navigate]);

  if (checking) {
    return (
      <div className="flex items-center justify-center h-screen bg-bg">
        <p className="text-text-muted">Loading...</p>
      </div>
    );
  }

  return <>{children}</>;
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/setup" element={<SetupWizard />} />
        <Route
          element={
            <SetupGuard>
              <AppLayout />
            </SetupGuard>
          }
        >
          <Route path="/" element={<SessionsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/:sessionId" element={<SessionDetailPage />} />
          <Route
            path="/:sessionId/:conversationId"
            element={<ConversationDetailPage />}
          />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
