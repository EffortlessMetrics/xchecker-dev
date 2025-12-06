Based on my analysis of the requirements and design documents for the demo-time-api specification, I'll now create a comprehensive implementation plan. The requirements document describes a RESTful API for time-related operations (current time retrieval, timezone conversion, event scheduling, and statistics), and the design document outlines 7 main components and 8 API endpoints.

# Implementation Plan

## Task Format

- [ ] 1. Set up project structure and core interfaces
  - Create directory structure for models, services, repositories, handlers, and middleware
  - Define core trait interfaces that establish system boundaries (TimeService, TimezoneService, EventScheduler, StatisticsService, ValidationService, AuthService, RateLimiter)
  - Create error types and error response structures
  - _Requirements: NFR3 (Authentication), NFR4 (Rate Limiting), NFR8 (Error Handling), NFR10 (Monitoring)_

- [ ] 2. Implement data models and validation
  
  - [ ] 2.1 Create core data model types and structures
    - Define Rust structs for TimeResponse, ConversionRequest, ConversionResponse, Event, Statistics, ErrorResponse
    - Implement serialization/deserialization with serde
    - _Requirements: Requirement 1 (Current Time), Requirement 2 (Timezone Conversion), Requirement 3 (Event Scheduling), Requirement 4 (Statistics), Requirement 5 (Time Format Validation)_
  
  - [ ] 2.2 Implement ValidationService with timezone and timestamp validation
    - Validate IANA timezone identifiers against chrono-tz database
    - Validate timestamp formats (ISO 8601, Unix epoch) and normalize to ISO 8601 with millisecond precision
    - Validate that scheduled event times are not in the past
    - Validate URL format for event callbacks
    - _Requirements: Requirement 5 (Validate Time Format), Requirement 3 (Schedule Time-Based Events)_

- [ ] 3. Implement authentication and rate limiting
  
  - [ ] 3.1 Implement AuthService for Bearer token validation
    - Parse and validate Bearer tokens from request headers
    - Implement 24-hour token expiration checking
    - Return appropriate 401 Unauthorized errors for invalid/expired tokens
    - _Requirements: NFR3 (Security - Authentication)_
  
  - [ ] 3.2 Implement RateLimiter with token bucket algorithm
    - Implement token bucket algorithm for 100 requests per minute per API key
    - Track request counts by API key
    - Return 429 Too Many Requests when rate limit exceeded
    - Include rate limit information in response headers (X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset)
    - _Requirements: NFR4 (Security - Rate Limiting)_

- [ ] 4. Implement core time and timezone services
  
  - [ ] 4.1 Implement TimeService for current time retrieval
    - Implement get_current_time() returning current UTC timestamp
    - Implement get_current_time_in_timezone() with optional 1-second caching
    - Validate timezone parameter (empty defaults to UTC)
    - _Requirements: Requirement 1 (Retrieve Current Time)_
  
  - [ ] 4.2 Implement TimezoneService for timezone conversions
    - Parse ISO 8601 and Unix epoch timestamps
    - Convert between timezones using chrono-tz
    - Handle DST transitions and document resolution strategy in response
    - Support both source and target timezone validation
    - _Requirements: Requirement 2 (Timezone Conversion), NFR5 (Data Accuracy)_

- [ ] 5. Implement event scheduling and management
  
  - [ ] 5.1 Implement EventScheduler with priority queue
    - Create Event struct with id, target_time, timezone, callback_url, created_at, status
    - Implement priority queue ordering by execution time
    - Validate target time is not in the past (return 400 Bad Request if past)
    - Return 201 Created with event ID on successful scheduling
    - _Requirements: Requirement 3 (Schedule Time-Based Events)_
  
  - [ ] 5.2 Implement event callback execution with retry logic
    - Trigger callbacks when event time arrives
    - Implement 30-second timeout for callback execution
    - Implement 3-retry strategy for failed callbacks
    - Log execution details with event context
    - _Requirements: Requirement 3 (Schedule Time-Based Events)_

- [ ] 6. Implement statistics service
  
  - [ ] 6.1 Create statistics data collection infrastructure
    - Track API request metrics: timestamp, method, endpoint, status code, response time
    - Aggregate by hour for date ranges ≤90 days, by day for >90 days
    - Calculate: average response time, peak usage hours, timezone distribution
    - _Requirements: Requirement 4 (Retrieve Time Statistics), NFR10 (Monitoring)_
  
  - [ ] 6.2 Implement statistics retrieval with access control
    - Implement GET /api/stats/history endpoint with date range parameters
    - Validate date range and apply granularity rules (hourly ≤90 days, daily >90 days)
    - Enforce admin-only access (403 Forbidden for non-admins)
    - Return 204 No Content when no data exists for range
    - _Requirements: Requirement 4 (Retrieve Time Statistics)_

- [ ] 7. Implement HTTP API handlers and middleware
  
  - [ ] 7.1 Create middleware pipeline
    - Implement authentication middleware that validates Bearer tokens on all routes
    - Implement rate limiting middleware that checks token bucket and sets response headers
    - Implement request logging middleware that emits structured JSON logs (timestamp, method, endpoint, status, response_time)
    - _Requirements: NFR3 (Authentication), NFR4 (Rate Limiting), NFR10 (Monitoring)_
  
  - [ ] 7.2 Implement handlers for time endpoints
    - Implement GET /api/time/current handler with optional timezone query parameter
    - Validate timezone parameter, return 400 for invalid timezone
    - Return TimeResponse with UTC timestamp and timezone-adjusted time
    - _Requirements: Requirement 1 (Retrieve Current Time)_
  
  - [ ] 7.3 Implement handlers for timezone conversion endpoint
    - Implement POST /api/time/convert handler accepting timestamp, source_timezone, target_timezone
    - Validate request body format, return 422 Unprocessable Entity for malformed JSON
    - Validate both timezones, return 400 for invalid timezones
    - Return ConversionResponse with converted timestamp and DST handling details
    - _Requirements: Requirement 2 (Timezone Conversion)_
  
  - [ ] 7.4 Implement handlers for event scheduling endpoints
    - Implement POST /api/events/schedule handler accepting event details and target_timestamp
    - Validate event data (timezone, callback URL), return 400 for past times
    - Return 201 Created with event ID and confirmation details
    - Implement internal event triggering mechanism and callback invocation
    - _Requirements: Requirement 3 (Schedule Time-Based Events)_
  
  - [ ] 7.5 Implement handlers for statistics endpoint
    - Implement GET /api/stats/history handler with start_date and end_date parameters
    - Apply granularity rules based on date range length
    - Return aggregated statistics or 204 No Content if no data
    - Enforce admin-only access
    - _Requirements: Requirement 4 (Retrieve Time Statistics)_

- [ ] 8. Implement error handling and response formatting
  
  - [ ] 8.1 Create error response formatter
    - Generate consistent ErrorResponse with error_code, http_status, message, request_id, timestamp
    - Implement error code mapping (e.g., E001_INVALID_TIMEZONE, E002_INVALID_TIMESTAMP, E003_PAST_EVENT_TIME)
    - Return JSON error responses with appropriate HTTP status codes
    - Include unique request IDs for debugging
    - _Requirements: NFR8 (Reliability - Error Handling)_
  
  - [ ] 8.2 Implement comprehensive error handling in all services
    - Handle validation errors → 400 Bad Request
    - Handle malformed JSON → 422 Unprocessable Entity
    - Handle authentication failures → 401 Unauthorized
    - Handle rate limit exceeded → 429 Too Many Requests
    - Handle permission denied → 403 Forbidden
    - Handle not found → 404 Not Found
    - _Requirements: NFR8 (Reliability - Error Handling)_

- [ ] 9. Wire up application and create main entry point
  
  - [ ] 9.1 Create application builder and dependency injection
    - Initialize all services (TimeService, TimezoneService, EventScheduler, StatisticsService, ValidationService, AuthService, RateLimiter)
    - Set up connection pools or state management
    - Create request context with initialized dependencies
    - _Requirements: All functional requirements_
  
  - [ ] 9.2 Configure HTTP server with routing and middleware
    - Register all middleware in correct order (logging, authentication, rate limiting)
    - Register all endpoint handlers
    - Configure TLS 1.2+ support for HTTPS
    - Set up graceful shutdown
    - _Requirements: NFR2 (Availability), NFR6 (Scalability), NFR9 (Compatibility)_
  
  - [ ] 9.3 Create main entry point and server startup
    - Parse configuration (port, log level, authentication settings)
    - Initialize application builder
    - Start HTTP server and listen for requests
    - Emit startup logs with version and configuration
    - _Requirements: All non-functional requirements_

- [ ]* 10. Write comprehensive test suites
  
  - [ ]* 10.1 Unit tests for validation service
    - Test valid/invalid IANA timezone identifiers
    - Test ISO 8601 timestamp parsing and normalization
    - Test Unix epoch conversion
    - Test ambiguous format detection (return 400)
    - Test millisecond precision normalization
    - _Requirements: Requirement 5 (Validate Time Format)_
  
  - [ ]* 10.2 Unit tests for time and timezone services
    - Test current time retrieval in various timezones
    - Test timezone conversions with DST transitions
    - Test default timezone (UTC) when not specified
    - Test 1-second caching behavior
    - _Requirements: Requirement 1 (Retrieve Current Time), Requirement 2 (Timezone Conversion)_
  
  - [ ]* 10.3 Unit tests for authentication and rate limiting
    - Test valid Bearer token acceptance
    - Test expired token rejection (401)
    - Test missing token rejection (401)
    - Test rate limit enforcement (429 after 100 requests)
    - Test rate limit header presence
    - _Requirements: NFR3 (Authentication), NFR4 (Rate Limiting)_
  
  - [ ]* 10.4 Unit tests for event scheduler
    - Test event creation with valid future time
    - Test rejection of past times (400)
    - Test priority queue ordering by execution time
    - Test event ID generation and return
    - Test callback timeout and retry logic
    - _Requirements: Requirement 3 (Schedule Time-Based Events)_
  
  - [ ]* 10.5 Unit tests for statistics service
    - Test hourly aggregation for ≤90 day ranges
    - Test daily aggregation for >90 day ranges
    - Test metric calculations (avg response time, peak hours, timezone distribution)
    - Test admin-only access control (403 for non-admins)
    - Test 204 No Content for empty results
    - _Requirements: Requirement 4 (Retrieve Time Statistics)_
  
  - [ ]* 10.6 Integration tests for API endpoints
    - Test GET /api/time/current with and without timezone parameter
    - Test 400 response for invalid timezone
    - Test POST /api/time/convert with valid/invalid timestamps and timezones
    - Test 422 response for malformed JSON
    - Test POST /api/events/schedule with past time rejection
    - Test GET /api/stats/history with date range and granularity
    - Test error response format consistency
    - Test rate limit and authentication on all endpoints
    - _Requirements: All functional requirements, NFR3, NFR4, NFR8, NFR10_
  
  - [ ]* 10.7 Contract/API tests
    - Test all endpoints return expected response schemas
    - Test all error responses have required fields (error_code, message, timestamp)
    - Test response status codes match requirements
    - Test rate limit headers present on all responses
    - _Requirements: NFR1 (Performance), NFR2 (Availability)_
  
  - [ ]* 10.8 Performance and load tests
    - Test p95 response time < 200ms under normal load
    - Test handling 5000 concurrent requests
    - Test rate limiting behavior under load
    - Measure response times for each endpoint
    - _Requirements: NFR1 (Performance), NFR6 (Scalability)_
  
  - [ ]* 10.9 Security tests
    - Test missing/malformed authentication headers
    - Test token expiration validation
    - Test rate limit bypass prevention
    - Test input validation for all endpoints
    - Test TLS 1.2+ enforcement
    - _Requirements: NFR3 (Authentication), NFR4 (Rate Limiting), NFR5 (Data Accuracy), NFR9 (Compatibility)_

---

This implementation plan breaks down the demo-time-api feature into 10 major task groups with 30+ specific coding tasks. Each task is mapped to specific functional and non-functional requirements, ensuring complete coverage. The plan follows an incremental, test-driven approach where services are built from the ground up, integrated into handlers, wired into the application, and then thoroughly tested.