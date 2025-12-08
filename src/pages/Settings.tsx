import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useTheme } from '@/hooks/useTheme';
import { useAppStore, type Theme } from '@/store';
import { ArrowLeft, Check, Monitor, Moon, Sun } from 'lucide-react';
import { useTranslation } from 'react-i18next';

const languages = [
  { code: 'en-US', name: 'English' },
  { code: 'pt-BR', name: 'Português (Brasil)' },
];

const themes: { value: Theme; icon: React.ComponentType<{ className?: string }>; labelKey: string }[] = [
  { value: 'light', icon: Sun, labelKey: 'settings.sections.appearance.themes.light' },
  { value: 'dark', icon: Moon, labelKey: 'settings.sections.appearance.themes.dark' },
  { value: 'system', icon: Monitor, labelKey: 'settings.sections.appearance.themes.system' },
];

interface SettingsProps {
  onBack: () => void;
}

export default function Settings({ onBack }: SettingsProps) {
  const { t, i18n } = useTranslation('common');
  const { settings, updateSettings } = useAppStore();
  const { theme, setTheme } = useTheme();

  const handleMinimizeToTrayChange = (checked: boolean) => {
    updateSettings({ minimizeToTray: checked });
  };

  return (
    <div className="h-screen bg-[var(--background)] text-[var(--foreground)] flex flex-col overflow-hidden transition-colors duration-200">
      {/* Header */}
      <div className="border-b border-[var(--border)] bg-[var(--background-secondary)]/80 backdrop-blur-sm">
        <div className="max-w-4xl mx-auto px-6 py-4">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="icon"
              onClick={onBack}
              className="h-9 w-9 rounded-lg"
            >
              <ArrowLeft className="w-4 h-4" />
            </Button>
            <div>
              <h1 className="text-xl font-semibold text-[var(--foreground)]">
                {t('settings.title')}
              </h1>
              <p className="text-xs text-[var(--foreground-muted)] mt-0.5">
                {t('settings.subtitle')}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Settings Content */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        <div className="max-w-4xl mx-auto px-6 py-6 space-y-4">
          {/* Appearance Settings */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base font-semibold">
                {t('settings.sections.appearance.title', 'Aparência')}
              </CardTitle>
              <CardDescription className="text-xs">
                {t('settings.sections.appearance.description', 'Personalize a aparência da aplicação')}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              <Label className="text-sm text-[var(--foreground-secondary)] mb-2 block">
                {t('settings.sections.appearance.themeLabel', 'Tema')}
              </Label>
              <div className="grid grid-cols-3 gap-2">
                {themes.map(({ value, icon: Icon, labelKey }) => (
                  <button
                    key={value}
                    onClick={() => setTheme(value)}
                    className={`
                      relative flex flex-col items-center gap-2 p-4 rounded-lg border transition-all
                      ${theme === value
                        ? 'bg-[var(--primary-muted)] border-[var(--primary)] text-[var(--primary)]'
                        : 'bg-[var(--surface)]/30 border-[var(--border-subtle)] hover:bg-[var(--surface)]/50 hover:border-[var(--border)] text-[var(--foreground-secondary)]'
                      }
                    `}
                  >
                    <Icon className="w-5 h-5" />
                    <span className="text-xs font-medium">
                      {t(labelKey, value.charAt(0).toUpperCase() + value.slice(1))}
                    </span>
                    {theme === value && (
                      <div className="absolute top-2 right-2">
                        <Check className="w-3.5 h-3.5" />
                      </div>
                    )}
                  </button>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Language Settings */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base font-semibold">
                {t('settings.sections.language.title')}
              </CardTitle>
              <CardDescription className="text-xs">
                {t('settings.sections.language.description')}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {languages.map((lang) => (
                <button
                  key={lang.code}
                  onClick={() => i18n.changeLanguage(lang.code)}
                  className={`
                    w-full flex items-center justify-between p-3 rounded-lg border transition-all
                    ${i18n.language === lang.code
                      ? 'bg-[var(--primary-muted)] border-[var(--primary)]/50'
                      : 'bg-[var(--surface)]/30 border-[var(--border-subtle)] hover:bg-[var(--surface)]/50 hover:border-[var(--border)]'
                    }
                  `}
                >
                  <span className={`text-sm font-medium ${i18n.language === lang.code ? 'text-[var(--primary)]' : 'text-[var(--foreground)]'}`}>
                    {lang.name}
                  </span>
                  {i18n.language === lang.code && (
                    <Check className="w-4 h-4 text-[var(--primary)]" />
                  )}
                </button>
              ))}
            </CardContent>
          </Card>

          {/* Application Behavior */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base font-semibold">
                {t('settings.sections.application.title')}
              </CardTitle>
              <CardDescription className="text-xs">
                {t('settings.sections.application.description')}
              </CardDescription>
            </CardHeader>
            <CardContent>
              {/* Minimize to Tray */}
              <div className="flex items-center justify-between space-x-4 p-3 rounded-lg bg-[var(--surface)]/30 border border-[var(--border-subtle)]">
                <div className="flex-1 space-y-0.5">
                  <Label htmlFor="minimize-tray" className="text-sm font-medium text-[var(--foreground)] cursor-pointer">
                    {t('settings.sections.application.minimizeToTray.label')}
                  </Label>
                  <p className="text-xs text-[var(--foreground-muted)] leading-relaxed">
                    {t('settings.sections.application.minimizeToTray.description')}
                  </p>
                </div>
                <Switch
                  id="minimize-tray"
                  checked={settings.minimizeToTray}
                  onCheckedChange={handleMinimizeToTrayChange}
                />
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
