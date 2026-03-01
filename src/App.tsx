import { BrowserRouter, Routes, Route } from "react-router-dom";
import Sidebar from "./components/layout/Sidebar";
import MainContent from "./components/layout/MainContent";
import SessionsPage from "./pages/SessionsPage";
import SessionDetailPage from "./pages/SessionDetailPage";
import ConversationDetailPage from "./pages/ConversationDetailPage";
import SettingsPage from "./pages/SettingsPage";

function AppLayout() {
  return (
    <div className="flex h-screen bg-bg">
      <Sidebar />
      <MainContent />
    </div>
  );
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/setup" element={<div>Setup Wizard</div>} />
        <Route element={<AppLayout />}>
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
