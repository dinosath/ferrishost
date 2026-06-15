import { Route, Routes, Navigate } from "@solidjs/router";
import { StorePage } from "./pages/StorePage";
import { ModuleDetail } from "./pages/ModuleDetail";
import { InstalledPage } from "./pages/InstalledPage";
import { SetupPage } from "./pages/SetupPage";

export default function App() {
  return (
    <div class="min-h-screen bg-gray-50">
      <Routes>
          <Route path="/" component={() => <Navigate href="/store" />} />
          <Route path="/store" component={StorePage} />
          <Route path="/store/:id" component={ModuleDetail} />
          <Route path="/installed" component={InstalledPage} />
          <Route path="/setup" component={SetupPage} />
      </Routes>
    </div>
  );
}
