"use client"

import { useSidebar } from "@/components/Sidebar/SidebarProvider";
import { useState, useEffect, useCallback, Suspense } from "react";
import { Transcript } from "@/types";
import PageContent from "./page-content";
import { useRouter, useSearchParams } from "next/navigation";
import Analytics from "@/lib/analytics";
import { LoaderIcon } from "lucide-react";
import { usePaginatedTranscripts } from "@/hooks/usePaginatedTranscripts";

interface MeetingDetailsResponse {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  transcripts: Transcript[];
  folder_path?: string;
}

function MeetingDetailsContent() {
  const searchParams = useSearchParams();
  const meetingId = searchParams.get('id');
  const { setCurrentMeeting, refetchMeetings } = useSidebar();
  const router = useRouter();
  const [meetingDetails, setMeetingDetails] = useState<MeetingDetailsResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const {
    metadata,
    segments,
    transcripts,
    isLoading: isLoadingTranscripts,
    isLoadingMore,
    hasMore,
    totalCount,
    loadedCount,
    loadMore,
    refetch,
    error: transcriptError,
  } = usePaginatedTranscripts({ meetingId: meetingId || '' });

  useEffect(() => {
    if (metadata && (!meetingId || meetingId === 'intro-call')) {
      return;
    }

    if (metadata) {
      setMeetingDetails({
        id: metadata.id,
        title: metadata.title,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        transcripts,
        folder_path: metadata.folder_path,
      });

      setCurrentMeeting({ id: metadata.id, title: metadata.title });
    }
  }, [metadata, transcripts, meetingId, setCurrentMeeting]);

  useEffect(() => {
    if (transcriptError) {
      console.error('Error loading transcripts:', transcriptError);
      setError(transcriptError);
    }
  }, [transcriptError]);

  const refreshMeeting = useCallback(async () => {
    if (!meetingId || meetingId === 'intro-call') {
      return;
    }

    await refetch();
  }, [meetingId, refetch]);

  useEffect(() => {
    setMeetingDetails(null);
    setError(null);
    setIsLoading(true);
  }, [meetingId]);

  useEffect(() => {
    if (!meetingId || meetingId === 'intro-call') {
      setError("No meeting selected");
      setIsLoading(false);
      Analytics.trackPageView('meeting_details');
      return;
    }

    setMeetingDetails(null);
    setError(null);
    setIsLoading(false);
  }, [meetingId]);

  if (error) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <p className="text-red-500 mb-4">{error}</p>
          <button
            onClick={() => router.push('/')}
            className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            Go Back
          </button>
        </div>
      </div>
    );
  }

  if ((isLoading || isLoadingTranscripts) || !meetingDetails) {
    return (
      <div className="flex items-center justify-center h-screen">
        <LoaderIcon className="animate-spin size-6" />
      </div>
    );
  }

  return (
    <PageContent
      meeting={meetingDetails}
      onRefetchTranscripts={async () => {
        await refreshMeeting();
        await refetchMeetings();
      }}
      segments={segments}
      hasMore={hasMore}
      isLoadingMore={isLoadingMore}
      totalCount={totalCount}
      loadedCount={loadedCount}
      onLoadMore={loadMore}
    />
  );
}

export default function MeetingDetails() {
  return (
    <Suspense fallback={
      <div className="flex items-center justify-center h-screen">
        <LoaderIcon className="animate-spin size-6" />
      </div>
    }>
      <MeetingDetailsContent />
    </Suspense>
  );
}
