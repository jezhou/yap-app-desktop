import { Outlet } from "react-router-dom";
import NavBar from "./NavBar";

export default function MainContent() {
  return (
    <div className="flex flex-col flex-1 min-w-0">
      <NavBar />
      <main className="flex-1 overflow-y-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}
