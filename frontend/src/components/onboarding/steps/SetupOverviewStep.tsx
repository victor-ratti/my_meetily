import React, { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { OnboardingContainer } from '../OnboardingContainer';
import { useOnboarding } from '@/contexts/OnboardingContext';

export function SetupOverviewStep() {
  const { goNext } = useOnboarding();
  const [isMac, setIsMac] = useState(false);

  useEffect(() => {
    const checkPlatform = async () => {
      try {
        const { platform } = await import('@tauri-apps/plugin-os');
        setIsMac(platform() === 'macos');
      } catch {
        setIsMac(navigator.userAgent.includes('Mac'));
      }
    };
    checkPlatform();
  }, []);

  return (
    <OnboardingContainer
      title="Setup Overview"
      description="Meetily needs a local transcription model before it can record and transcribe meetings."
      step={2}
      totalSteps={isMac ? 4 : 3}
    >
      <div className="flex flex-col items-center space-y-10">
        <div className="w-full max-w-md bg-white rounded-lg border border-gray-200 p-4">
          <div className="flex items-start gap-4 p-1">
            <div className="flex-1 ml-1">
              <h3 className="font-medium text-gray-900">
                Step 1 : Download Transcription Engine
              </h3>
            </div>
          </div>
        </div>

        <div className="w-full max-w-xs space-y-4">
          <Button
            onClick={goNext}
            className="w-full h-11 bg-gray-900 hover:bg-gray-800 text-white"
          >
            Let's Go
          </Button>
          <div className="text-center">
            <a
              href="https://github.com/Zackriya-Solutions/meeting-minutes"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-gray-600 hover:underline"
            >
              Report issues on GitHub
            </a>
          </div>
        </div>
      </div>
    </OnboardingContainer>
  );
}
