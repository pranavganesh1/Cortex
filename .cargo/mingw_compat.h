#ifndef MINGW_COMPAT_H
#define MINGW_COMPAT_H

#include <winsock2.h>
#include <windows.h>
#include <winerror.h>
#include <winbase.h>

// Definitions for well known SIDs
#ifndef WinBuiltinAdministratorsSid
typedef enum {
    WinNullSid = 0,
    WinWorldSid = 1,
    WinLocalSid = 2,
    WinCreatorOwnerSid = 3,
    WinCreatorGroupSid = 4,
    WinCreatorOwnerServerSid = 5,
    WinCreatorGroupServerSid = 6,
    WinNtAuthoritySid = 7,
    WinDialupSid = 8,
    WinNetworkSid = 9,
    WinBatchSid = 10,
    WinInteractiveSid = 11,
    WinServiceSid = 12,
    WinAnonymousSid = 13,
    WinProxySid = 14,
    WinEnterpriseControllersSid = 15,
    WinSelfSid = 16,
    WinAuthenticatedUserSid = 17,
    WinRestrictedCodeSid = 18,
    WinTerminalServerSid = 19,
    WinRemoteLogonIdSid = 20,
    WinLogonIdsSid = 21,
    WinLocalSystemSid = 22,
    WinLocalServiceSid = 23,
    WinNetworkServiceSid = 24,
    WinBuiltinDomainSid = 25,
    WinBuiltinAdministratorsSid = 26,
    WinBuiltinUsersSid = 27,
    WinBuiltinGuestsSid = 28,
    WinBuiltinPowerUsersSid = 29,
    WinBuiltinAccountOperatorsSid = 30,
    WinBuiltinSystemOperatorsSid = 31,
    WinBuiltinPrintOperatorsSid = 32,
    WinBuiltinBackupOperatorsSid = 33,
    WinBuiltinReplicatorSid = 34,
    WinBuiltinPreWindows2000CompatibleAccessSid = 35,
    WinBuiltinRemoteDesktopUsersSid = 36,
    WinBuiltinNetworkConfigurationOperatorsSid = 37,
    WinAccountAdministratorSid = 38,
    WinAccountGuestSid = 39,
    WinAccountKrbtgtSid = 40,
    WinAccountDomainAdminsSid = 41,
    WinAccountDomainUsersSid = 42,
    WinAccountDomainGuestsSid = 43,
    WinAccountComputersSid = 44,
    WinAccountControllersSid = 45,
    WinAccountCertAdminsSid = 46,
    WinAccountSchemaAdminsSid = 47,
    WinAccountEnterpriseAdminsSid = 48,
    WinAccountPolicyAdminsSid = 49,
    WinAccountRasAndIasServersSid = 50,
    WinNTLMAuthenticationSid = 51,
    WinDigestAuthenticationSid = 52,
    WinSChannelAuthenticationSid = 53,
    WinThisOrganizationSid = 54,
    WinOtherOrganizationSid = 55,
    WinBuiltinIncomingForestTrustBuildersSid = 56,
    WinBuiltinPerfMonitoringUsersSid = 57,
    WinBuiltinPerfLoggingUsersSid = 58,
    WinBuiltinAuthorizationAccessSid = 59,
    WinBuiltinTerminalServerLicenseServersSid = 60,
    WinBuiltinDCOMUsersSid = 61,
    WinBuiltinIUsersSid = 62,
    WinIUserSid = 63,
    WinBuiltinCryptoOperatorsSid = 64,
    WinUntrustedLabelSid = 65,
    WinLowLabelSid = 66,
    WinMediumLabelSid = 67,
    WinHighLabelSid = 68,
    WinSystemLabelSid = 69,
    WinWriteRestrictedCodeSid = 70,
    WinCreatorOwnerGroupsSid = 71,
    WinCreatorGroupGroupsSid = 72,
    WinEnterpriseReadonlyControllersSid = 73,
    WinAccountReadonlyControllersSid = 74,
    WinAccountSpecialSid = 75,
    WinBuiltinEventLogReadersSid = 76,
    WinNewEnterpriseReadonlyControllersSid = 77,
    WinBuiltinCertSvcDComAccessGroupSid = 78,
    WinMediumPlusLabelSid = 79,
    WinLocalLogonSid = 80,
    WinConsoleLogonSid = 81,
    WinThisOrganizationCertificateSid = 82,
    WinApplicationPackageAuthoritySid = 83,
    WinBuiltinAnyPackageSid = 84,
    WinLocalAccountSid = 85,
    WinLocalAccountAndAdministratorSid = 86,
    WinThreadAllowUserModeAPCSid = 87,
    WinUserModeAPCSid = 88,
    WinHighRestrictedLabelSid = 89
} WELL_KNOWN_SID_TYPE;
#endif

// Declare IsWellKnownSid with __attribute__((__stdcall__))
BOOL __attribute__((__stdcall__)) IsWellKnownSid(PSID pSid, WELL_KNOWN_SID_TYPE WellKnownSidType);

// Declare GetTickCount64 with __attribute__((__stdcall__))
ULONGLONG __attribute__((__stdcall__)) GetTickCount64(VOID);

// Definitions for pollfd and flags
#ifndef POLLIN
struct pollfd {
    int fd;
    short events;
    short revents;
};
#define POLLIN      0x0100
#define POLLPRI     0x0200
#define POLLOUT     0x0010
#define POLLERR     0x0001
#define POLLHUP     0x0002
#define POLLNVAL    0x0004
#endif

// Declare WSAPoll with __attribute__((__stdcall__))
int __attribute__((__stdcall__)) WSAPoll(struct pollfd *fdArray, ULONG fds, INT timeout);

// POSIX error codes
#ifndef ETIMEDOUT
#define ETIMEDOUT WSAETIMEDOUT
#endif

#ifndef EINPROGRESS
#define EINPROGRESS WSAEINPROGRESS
#endif

#ifndef EWOULDBLOCK
#define EWOULDBLOCK WSAEWOULDBLOCK
#endif

// Windows errors
#ifndef ERROR_INSUFFICIENT_BUFFER
#define ERROR_INSUFFICIENT_BUFFER 122L
#endif

// FLS declarations
typedef VOID (__attribute__((__stdcall__)) *PFLS_CALLBACK_FUNCTION)(PVOID);
DWORD __attribute__((__stdcall__)) FlsAlloc(PFLS_CALLBACK_FUNCTION lpCallback);
BOOL __attribute__((__stdcall__)) FlsFree(DWORD dwFlsIndex);
BOOL __attribute__((__stdcall__)) FlsSetValue(DWORD dwFlsIndex, PVOID lpFlsData);
PVOID __attribute__((__stdcall__)) FlsGetValue(DWORD dwFlsIndex);

// InterlockedAdd macro for 32-bit MinGW
#ifndef InterlockedAdd
#define InterlockedAdd(a, v) (InterlockedExchangeAdd((LONG volatile *)(a), (LONG)(v)) + (LONG)(v))
#endif

#endif // MINGW_COMPAT_H
