import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';

import { AppShell } from './layout/AppShell';
import { LoginPage } from './pages/LoginPage';
import { InsightsPage } from './pages/InsightsPage';
import { NotFoundPage } from './pages/NotFoundPage';
import { RegisterPage } from './pages/RegisterPage';
import { TeamPage } from './pages/TeamPage';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<Navigate to="/insights" replace />} />
          <Route path="insights" element={<InsightsPage />} />
          <Route path="team" element={<TeamPage />} />
          <Route path="login" element={<LoginPage />} />
          <Route path="register" element={<RegisterPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
