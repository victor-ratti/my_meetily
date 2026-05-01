'use client';

import React, { createContext, useContext, useState, useEffect } from 'react';
import { usePathname, useRouter } from 'next/navigation';
import Analytics from '@/lib/analytics';
import { invoke } from '@tauri-apps/api/core';
import { useRecordingState } from '@/contexts/RecordingStateContext';

interface SidebarItem {
  id: string;
  title: string;
  type: 'folder' | 'file';
  children?: SidebarItem[];
}

export interface CurrentMeeting {
  id: string;
  title: string;
}

interface TranscriptSearchResult {
  id: string;
  title: string;
  matchContext: string;
  timestamp: string;
}

interface SidebarContextType {
  currentMeeting: CurrentMeeting | null;
  setCurrentMeeting: (meeting: CurrentMeeting | null) => void;
  sidebarItems: SidebarItem[];
  isCollapsed: boolean;
  toggleCollapse: () => void;
  meetings: CurrentMeeting[];
  setMeetings: (meetings: CurrentMeeting[]) => void;
  isMeetingActive: boolean;
  setIsMeetingActive: (active: boolean) => void;
  handleRecordingToggle: () => void;
  searchTranscripts: (query: string) => Promise<void>;
  searchResults: TranscriptSearchResult[];
  isSearching: boolean;
  setServerAddress: (address: string) => void;
  serverAddress: string;
  transcriptServerAddress: string;
  setTranscriptServerAddress: (address: string) => void;
  refetchMeetings: () => Promise<void>;
}

const SidebarContext = createContext<SidebarContextType | null>(null);

export const useSidebar = () => {
  const context = useContext(SidebarContext);
  if (!context) {
    throw new Error('useSidebar must be used within a SidebarProvider');
  }
  return context;
};

export function SidebarProvider({ children }: { children: React.ReactNode }) {
  const [currentMeeting, setCurrentMeeting] = useState<CurrentMeeting | null>({ id: 'intro-call', title: '+ New Call' });
  const [isCollapsed, setIsCollapsed] = useState(true);
  const [meetings, setMeetings] = useState<CurrentMeeting[]>([]);
  const [sidebarItems, setSidebarItems] = useState<SidebarItem[]>([]);
  const [isMeetingActive, setIsMeetingActive] = useState(false);
  const [searchResults, setSearchResults] = useState<TranscriptSearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [serverAddress, setServerAddress] = useState('');
  const [transcriptServerAddress, setTranscriptServerAddress] = useState('');

  const { isRecording } = useRecordingState();
  const pathname = usePathname();
  const router = useRouter();

  const fetchMeetings = React.useCallback(async () => {
    if (!serverAddress) {
      return;
    }

    try {
      const meetings = await invoke('api_get_meetings') as Array<{ id: string, title: string }>;
      setMeetings(meetings.map((meeting: any) => ({
        id: meeting.id,
        title: meeting.title,
      })));
      Analytics.trackBackendConnection(true);
    } catch (error) {
      console.error('Error fetching meetings:', error);
      setMeetings([]);
      Analytics.trackBackendConnection(false, error instanceof Error ? error.message : 'Unknown error');
    }
  }, [serverAddress]);

  useEffect(() => {
    fetchMeetings();
  }, [fetchMeetings]);

  useEffect(() => {
    setServerAddress('http://localhost:5167');
    setTranscriptServerAddress('http://127.0.0.1:8178/stream');
  }, []);

  const baseItems = React.useMemo<SidebarItem[]>(() => [
    {
      id: 'meetings',
      title: 'Transcripts',
      type: 'folder',
      children: meetings.map(meeting => ({ id: meeting.id, title: meeting.title, type: 'file' as const })),
    },
  ], [meetings]);

  const toggleCollapse = () => {
    setIsCollapsed(!isCollapsed);
  };

  useEffect(() => {
    if (pathname === '/') {
      setCurrentMeeting({ id: 'intro-call', title: '+ New Call' });
    }
  }, [pathname]);

  useEffect(() => {
    setSidebarItems(baseItems);
  }, [baseItems]);

  const handleRecordingToggle = () => {
    if (!isRecording) {
      if (pathname === '/') {
        window.dispatchEvent(new CustomEvent('start-recording-from-sidebar'));
      } else {
        sessionStorage.setItem('autoStartRecording', 'true');
        router.push('/');
      }

      Analytics.trackButtonClick('start_recording', 'sidebar');
    }
  };

  const searchTranscripts = async (query: string) => {
    if (!query.trim()) {
      setSearchResults([]);
      return;
    }

    try {
      setIsSearching(true);
      const results = await invoke('api_search_transcripts', { query }) as TranscriptSearchResult[];
      setSearchResults(results);
    } catch (error) {
      console.error('Error searching transcripts:', error);
      setSearchResults([]);
    } finally {
      setIsSearching(false);
    }
  };

  return (
    <SidebarContext.Provider value={{
      currentMeeting,
      setCurrentMeeting,
      sidebarItems,
      isCollapsed,
      toggleCollapse,
      meetings,
      setMeetings,
      isMeetingActive,
      setIsMeetingActive,
      handleRecordingToggle,
      searchTranscripts,
      searchResults,
      isSearching,
      setServerAddress,
      serverAddress,
      transcriptServerAddress,
      setTranscriptServerAddress,
      refetchMeetings: fetchMeetings,
    }}>
      {children}
    </SidebarContext.Provider>
  );
}
