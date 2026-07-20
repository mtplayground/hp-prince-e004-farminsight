import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';

import { AppShell } from './layout/AppShell';
import { InsightsPage } from './pages/InsightsPage';
import { NotFoundPage } from './pages/NotFoundPage';
import { TeamPage } from './pages/TeamPage';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<Navigate to="/insights" replace />} />
          <Route path="insights" element={<InsightsPage />} />
          <Route path="team" element={<TeamPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
