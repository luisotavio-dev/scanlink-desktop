import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useAppStore } from '@/store';
import { ArrowLeft, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';

const languages = [
  { code: 'en-US', name: 'English' },
  { code: 'pt-BR', name: 'Português (Brasil)' },
];

interface SettingsProps {
  onBack: () => void;
}

export default function Settings({ onBack }: SettingsProps) {
  const { t, i18n } = useTranslation('common');
  const { settings, updateSettings } = useAppStore();

  const handleMinimizeToTrayChange = (checked: boolean) => {
    updateSettings({ minimizeToTray: checked });
  };

  return (
    <div className="h-screen bg-[#0f1419] text-white flex flex-col overflow-hidden">
      {/* Header */}
      <div className="border-b border-slate-800/50 bg-[#1a1f2e]/80 backdrop-blur-sm">
        <div className="max-w-4xl mx-auto px-6 py-4">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="icon"
              onClick={onBack}
              className="h-9 w-9 hover:bg-slate-800/60 rounded-lg"
            >
              <ArrowLeft className="w-4 h-4" />
            </Button>
            <div>
              <h1 className="text-xl font-semibold text-white">
                {t('settings.title')}
              </h1>
              <p className="text-xs text-slate-400 mt-0.5">
                {t('settings.subtitle')}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Settings Content */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        <div className="max-w-4xl mx-auto px-6 py-6 space-y-4">
          {/* Language Settings */}
          <Card className="bg-[#1a1f2e]/60 border-slate-800/50 shadow-xl">
            <CardHeader className="pb-3">
              <CardTitle className="text-base font-semibold text-white">
                {t('settings.sections.language.title')}
              </CardTitle>
              <CardDescription className="text-xs text-slate-400">
                {t('settings.sections.language.description')}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {languages.map((lang) => (
                <button
                  key={lang.code}
                  onClick={() => i18n.changeLanguage(lang.code)}
                  className="w-full flex items-center justify-between p-3 rounded-lg bg-slate-800/30 border border-slate-700/30 hover:bg-slate-800/50 hover:border-slate-600/50 transition-all"
                >
                  <span className="text-sm font-medium text-slate-200">{lang.name}</span>
                  {i18n.language === lang.code && (
                    <Check className="w-4 h-4 text-blue-400" />
                  )}
                </button>
              ))}
            </CardContent>
          </Card>

          {/* Application Behavior */}
          <Card className="bg-[#1a1f2e]/60 border-slate-800/50 shadow-xl">
            <CardHeader className="pb-3">
              <CardTitle className="text-base font-semibold text-white">
                {t('settings.sections.application.title')}
              </CardTitle>
              <CardDescription className="text-xs text-slate-400">
                {t('settings.sections.application.description')}
              </CardDescription>
            </CardHeader>
            <CardContent>
              {/* Minimize to Tray */}
              <div className="flex items-center justify-between space-x-4 p-3 rounded-lg bg-slate-800/30 border border-slate-700/30">
                <div className="flex-1 space-y-0.5">
                  <Label htmlFor="minimize-tray" className="text-sm font-medium text-slate-200 cursor-pointer">
                    {t('settings.sections.application.minimizeToTray.label')}
                  </Label>
                  <p className="text-xs text-slate-400 leading-relaxed">
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
